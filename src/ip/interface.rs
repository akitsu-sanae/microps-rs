use std::error::Error;
use std::sync::{Arc, Mutex};

use crate::{
    arp, buffer, ethernet,
    ip::{self, dgram, route},
    packet,
    protocol::ProtocolType,
    util,
};

#[derive(Debug)]
pub struct InterfaceImpl {
    pub device: ethernet::Device,
    pub unicast: ip::Addr,
    pub netmask: ip::Addr,
    pub gateway: ip::Addr,
}

#[derive(Debug, Clone)]
pub struct Interface(pub Arc<Mutex<InterfaceImpl>>);

impl Interface {
    pub fn new(
        device: ethernet::Device,
        unicast: ip::Addr,
        netmask: ip::Addr,
        gateway: ip::Addr,
    ) -> Interface {
        let interface = Interface(Arc::new(Mutex::new(InterfaceImpl {
            device: device,
            unicast: unicast,
            netmask: netmask,
            gateway: gateway,
        })));
        let network = unicast.apply_mask(&netmask);
        route::add(route::Route {
            network: network,
            netmask: netmask,
            nexthop: None,
            interface: interface.clone(),
        });
        route::add(route::Route {
            network: ip::ADDR_ANY,
            netmask: ip::ADDR_ANY,
            nexthop: Some(gateway),
            interface: interface.clone(),
        });
        interface
    }
    pub fn tx(
        &self,
        protocol: ProtocolType,
        mut packet: buffer::Buffer,
        dst: &ip::Addr,
    ) -> Result<(), Box<dyn Error>> {
        let (nexthop, interface, src) = if dst == &ip::ADDR_BROADCAST {
            (None, self.clone(), None)
        } else {
            match route::lookup(None, dst.clone()) {
                None => return Err(util::RuntimeError::new("ip no route to host".to_string())),
                Some(route) => {
                    let nexthop = Some(route.nexthop.unwrap_or(dst.clone()));
                    let interface = route.interface;
                    let src = Some(self.0.lock().unwrap().unicast.clone());
                    (nexthop, interface, src)
                }
            }
        };
        let id = generate_id();

        let mut segment_len: u16;
        let mut done: u16 = 0;
        while !packet.0.is_empty() {
            segment_len = ::std::cmp::min(
                packet.0.len() as u16,
                ethernet::PAYLOAD_SIZE_MAX as u16 - ip::dgram::HEADER_MIN_SIZE as u16,
            );
            let flag: u16 = if segment_len < packet.0.len() as u16 {
                0x2000
            } else {
                0x0000
            };
            let offset = flag | (done >> 3) & 0x1fff;
            let segment = packet.head(segment_len as usize);
            interface.tx_core(protocol, segment, src, dst.clone(), nexthop, id, offset)?;
            done += segment_len as u16;
        }
        Ok(())
    }

    fn tx_core(
        &self,
        type_: ProtocolType,
        buf: buffer::Buffer,
        src: Option<ip::Addr>,
        dst: ip::Addr,
        nexthop: Option<ip::Addr>,
        id: u16,
        offset: u16,
    ) -> Result<(), Box<dyn Error>> {
        let mut dgram = dgram::Dgram {
            version_header_length: (ip::VERSION << 4) | (ip::dgram::HEADER_LEN >> 2),
            type_of_service: 0,
            len: ip::dgram::HEADER_LEN as u16 + buf.0.len() as u16,
            id: id,
            offset: offset,
            time_to_live: 0xff,
            protocol: type_,
            checksum: 0,
            src: match src {
                Some(src) => src,
                None => {
                    let impl_ = self.0.lock().unwrap();
                    impl_.unicast
                }
            },
            dst: dst,
            payload: buffer::Buffer::empty(),
        };
        use packet::Packet;
        dgram.checksum = util::calc_checksum(dgram.clone().to_buffer(), 0); // TODO: remove `clone` if possible
        dgram.payload = buf;
        self.tx_device(dgram.to_buffer(), &nexthop)
    }

    pub fn tx_device(
        &self,
        data: buffer::Buffer,
        dst: &Option<ip::Addr>,
    ) -> Result<(), Box<dyn Error>> {
        use ethernet::DeviceFlags;
        let mac_addr = if DeviceFlags::BROADCAST & DeviceFlags::NOARP == DeviceFlags::EMPTY {
            match dst {
                Some(dst) => match arp::resolve(&self, *dst, data.clone())? {
                    // TODO: remove if possible
                    Some(addr) => addr,
                    None => return Ok(()),
                },
                None => {
                    let interface = self.0.lock().unwrap();
                    let device = interface.device.0.lock().unwrap();
                    device.broadcast_addr
                }
            }
        } else {
            ethernet::MacAddr::empty()
        };
        let interface = self.0.lock().unwrap();
        interface.device.tx(ethernet::Type::Ip, data, mac_addr)
    }

    pub fn reconfigure(&mut self, addr: ip::Addr, netmask: ip::Addr, gateway: Option<ip::Addr>) -> Result<(), Box<dyn Error>> {
        route::delete(self);
        let mut interface = self.0.lock().unwrap();
        interface.unicast = addr;
        let network = interface.unicast.apply_mask(&interface.netmask);
        route::add(route::Route {
            network: network,
            netmask: netmask,
            nexthop: Some(ip::ADDR_ANY),
            interface: self.clone(),
        });
        if let Some(gateway) = gateway {
            route::add(route::Route {
                network: ip::ADDR_ANY,
                netmask: ip::ADDR_ANY,
                nexthop: Some(gateway),
                interface: self.clone(),
            });
        }
        Ok(())
    }
}

fn generate_id() -> u16 {
    let mut id_counter = ID_COUNTER.lock().unwrap();
    let ret = *id_counter;
    *id_counter += 1;
    ret
}

lazy_static! {
    static ref ID_COUNTER: Mutex<u16> = Mutex::new(128);
}
