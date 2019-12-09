use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use crate::{
    arp, ethernet, frame, icmp, ip,
    protocol::{Protocol, ProtocolType},
    util,
};

mod fragment;
mod route;

pub const VERSION: u8 = 4;

pub const HEADER_MIN_SIZE: usize = 20;
pub const HEADER_MAX_SIZE: usize = 60;
pub const PAYLOAD_MAX_SIZE: usize = 65535 - HEADER_MIN_SIZE;

pub const ADDR_LEN: usize = 4;
const ADDR_BROADCAST: frame::IpAddr = frame::IpAddr([255; ADDR_LEN]);

const HEADER_LEN: u8 = 1+1+2+2+2+1+1+2+4+4+1;

#[derive(Debug, Clone)]
pub struct Dgram {
    pub version_header_length: u8,
    pub type_of_service: u8,
    pub len: u16,
    pub id: u16,
    pub offset: u16,
    pub time_to_live: u8,
    pub protocol: ProtocolType,
    pub checksum: u16,
    pub src: frame::IpAddr,
    pub dst: frame::IpAddr,
    pub payload: frame::Bytes,
}

impl Dgram {
    pub fn dump(&self) {
        eprintln!("version, header length: {}", self.version_header_length);
        eprintln!("type of service: {}", self.type_of_service);
        eprintln!("len: {}", self.len);
        eprintln!("id: {}", self.id);
        eprintln!("offset: {}", self.offset);
        eprintln!("time_to_live: {}", self.time_to_live);
        eprintln!("protocol: {}", self.protocol);
        eprintln!("checksum: {}", self.checksum);
        eprintln!("src: {}", self.src);
        eprintln!("dst: {}", self.dst);
        eprintln!("payload: {}", self.payload);
    }
}

impl frame::Frame for Dgram {
    fn from_bytes(mut bytes: frame::Bytes) -> Result<Box<Self>, Box<dyn Error>> {
        let version_header_length = bytes.pop_u8("vhl")?;
        let type_of_service = bytes.pop_u8("tos")?;
        let len = bytes.pop_u16("length")?;
        let id = bytes.pop_u16("id")?;
        let offset = bytes.pop_u16("flags and fragment offset")?;
        let time_to_live = bytes.pop_u8("ttl")?;
        let protocol = bytes.pop_u8("protocol")?;
        let protocol = ProtocolType::from_u8(protocol).ok_or(util::RuntimeError::new(format!(
            "{} can not be Protocol Family",
            protocol
        )))?;
        let checksum = bytes.pop_u16("checksum")?;
        let src = bytes.pop_ip_addr("src")?;
        let dst = bytes.pop_ip_addr("dst")?;
        let payload = bytes;

        Ok(Box::new(Dgram {
            version_header_length: version_header_length,
            type_of_service: type_of_service,
            len: len,
            id: id,
            offset: offset,
            time_to_live: time_to_live,
            protocol: protocol,
            checksum: checksum,
            src: src,
            dst: dst,
            payload: payload,
        }))
    }

    fn to_bytes(self) -> frame::Bytes {
        let mut bytes = frame::Bytes::new(HEADER_MAX_SIZE);
        bytes.push_u8(self.version_header_length);
        bytes.push_u8(self.type_of_service);
        bytes.push_u16(self.len);
        bytes.push_u16(self.id);
        bytes.push_u16(self.offset);
        bytes.push_u8(self.time_to_live);
        bytes.push_u8(self.protocol as u8);
        bytes.push_u16(self.checksum);
        bytes.push_ip_addr(self.src);
        bytes.push_ip_addr(self.dst);
        bytes.append(self.payload);

        bytes
    }
}

#[derive(Debug)]
pub struct InterfaceImpl {
    pub device: ethernet::Device,
    pub unicast: frame::IpAddr,
    pub netmask: frame::IpAddr,
    pub gateway: frame::IpAddr,
}

#[derive(Debug, Clone)]
pub struct Interface(pub Arc<Mutex<InterfaceImpl>>);

impl Interface {
    pub fn new(inner: InterfaceImpl) -> Interface {
        Interface(Arc::new(Mutex::new(inner)))
    }
    pub fn tx(
        &self,
        protocol: ProtocolType,
        mut packet: frame::Bytes,
        dst: &frame::IpAddr,
    ) -> Result<(), Box<dyn Error>> {
        let (nexthop, interface, src) = if dst == &ADDR_BROADCAST {
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

        let mut segment_len: u16 = 0;
        let mut done: u16 = 0;
        while !packet.0.is_empty() {
            segment_len = ::std::cmp::min(
                packet.0.len() as u16,
                ethernet::PAYLOAD_SIZE_MAX as u16 - ip::HEADER_MIN_SIZE as u16,
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
        buf: frame::Bytes,
        src: Option<frame::IpAddr>,
        dst: frame::IpAddr,
        nexthop: Option<frame::IpAddr>,
        id: u16,
        offset: u16,
    ) -> Result<(), Box<dyn Error>> {

        let mut dgram = Dgram {
            version_header_length: (VERSION << 4) | (HEADER_LEN >> 2),
            type_of_service: 0,
            len:HEADER_LEN as u16 + buf.0.len() as u16,
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
                },
            },
            dst : dst,
            payload: frame::Bytes::empty(),
        };
        use frame::Frame;
        dgram.checksum = util::calc_checksum(dgram.clone().to_bytes(), 0); // TODO: remove `clone` if possible
        dgram.payload = buf;
        tx_device(self, dgram.to_bytes(), &nexthop)
    }
}

fn generate_id() -> u16 {
    let mut id_counter = ID_COUNTER.lock().unwrap();
    let ret = *id_counter;
    *id_counter += 1;
    ret
}

use std::sync::atomic::{AtomicBool, Ordering};
lazy_static! {
    static ref IS_FORWARDING: AtomicBool = AtomicBool::new(false);
    static ref PROTOCOLS: Mutex<Vec<Arc<dyn Protocol + Send + Sync>>> =
        Mutex::new(vec![icmp::IcmpProtocol::new()]);
    static ref ID_COUNTER: Mutex<u16> = Mutex::new(128);
}

pub fn set_is_forwarding(b: bool) {
    IS_FORWARDING.store(b, Ordering::Relaxed);
}

fn forward_process(mut dgram: Dgram, interface: &ip::Interface) -> Result<(), Box<dyn Error>> {
    use frame::Frame;
    if dgram.time_to_live != 1 {
        let src = dgram.src;
        icmp::tx(
            interface,
            icmp::Type::TimeExceeded,
            icmp::Code::Exceeded(icmp::CodeExceeded::Ttl),
            0,
            dgram.to_bytes(),
            &src,
        )?;
        return Err(util::RuntimeError::new(format!("time exceeded")));
    }
    let route = match route::lookup(Some(interface), dgram.dst) {
        Some(route) => route,
        None => {
            let src = dgram.src;
            icmp::tx(
                interface,
                icmp::Type::DestUnreach,
                icmp::Code::Unreach(icmp::CodeUnreach::Net),
                0,
                dgram.to_bytes(),
                &src,
            )?;
            return Err(util::RuntimeError::new(format!("destination unreach")));
        }
    };
    {
        let route_interface = route.interface.0.lock().unwrap();
        let route_device = route_interface.device.clone();
        if route_interface.unicast == dgram.dst {
            ip::rx(dgram.to_bytes(), &route_device)?;
            return Ok(());
        }
    }
    if dgram.offset & 0x4000 != 0 && dgram.payload.0.len() > ethernet::PAYLOAD_SIZE_MAX {
        let src = dgram.src;
        return icmp::tx(
            interface,
            icmp::Type::DestUnreach,
            icmp::Code::Unreach(icmp::CodeUnreach::FragmentNeeded),
            0,
            dgram.to_bytes(),
            &src,
        );
        // return Err(util::RuntimeError::new(format!("destination unreach")));
    }
    dgram.time_to_live -= 1;
    let sum = dgram.checksum;
    dgram.checksum = util::calc_checksum(
        dgram
            .payload
            .head(((dgram.version_header_length & 0x0f) as usize) << 2),
        (u16::max_value() - dgram.checksum) as u32,
    );
    let ret = tx_device(
        &route.interface,
        dgram.clone().to_bytes(), // TODO: remove clone if possible
        &match route.nexthop {
            Some(next) => Some(next),
            None => Some(dgram.src),
        },
    );
    match ret {
        Ok(()) => Ok(()),
        Err(_) => {
            // restore original IP header
            dgram.time_to_live += 1;
            dgram.checksum = sum;

            let src = dgram.src;

            icmp::tx(
                interface,
                icmp::Type::DestUnreach,
                match route.nexthop {
                    Some(_) => icmp::Code::Unreach(icmp::CodeUnreach::Net),
                    None => icmp::Code::Unreach(icmp::CodeUnreach::Host),
                },
                0,
                dgram.to_bytes(),
                &src,
            )
        }
    }
}

pub fn rx(
    dgram: frame::Bytes,
    device: &ethernet::Device,
) -> Result<Option<JoinHandle<()>>, Box<dyn Error>> {
    use frame::Frame;
    let dgram = Dgram::from_bytes(dgram)?;
    let device = device.0.lock().unwrap();
    let interface = device
        .interface
        .as_ref()
        .ok_or(util::RuntimeError::new(format!(
            "device `{}` has not ip interface.",
            device.name
        )))?;
    let (unicast, broadcast) = {
        let interface = interface.0.lock().unwrap();
        let network = interface.unicast.apply_mask(&interface.netmask);
        let broadcast = network | !interface.netmask;
        (interface.unicast.clone(), broadcast)
    };
    if dgram.dst != unicast && dgram.dst != broadcast && dgram.dst != ADDR_BROADCAST {
        /* forward to other host */
        if IS_FORWARDING.load(Ordering::SeqCst) {
            forward_process(*dgram, interface)?;
        }
        return Ok(None);
    }
    dgram.dump();

    let (src, dst, protocol_type) = (dgram.src, dgram.dst, dgram.protocol);
    let payload = if dgram.offset & 0x2000 != 0 || dgram.offset & 0x1ff != 0 {
        let fragment = fragment::process(*dgram)?;
        fragment.data
    } else {
        dgram.payload
    };
    let protocols = PROTOCOLS.lock().unwrap();
    for protocol in protocols.iter() {
        if protocol.type_() == protocol_type {
            protocol.handler(payload, src, dst, interface)?;
            return Ok(None);
        }
    }
    Err(util::RuntimeError::new(format!("no suitable protocol")))
}

pub fn tx_device(
    interface: &Interface,
    data: frame::Bytes,
    dst: &Option<frame::IpAddr>,
) -> Result<(), Box<dyn Error>> {
    use ethernet::DeviceFlags;
    let mac_addr = if DeviceFlags::BROADCAST & DeviceFlags::NOARP == DeviceFlags::EMPTY {
        match dst {
            Some(dst) => match arp::resolve(interface, *dst, data.clone())? {
                // TODO: remove if possible
                Some(addr) => addr,
                None => return Ok(()),
            },
            None => {
                let interface = interface.0.lock().unwrap();
                let device = interface.device.0.lock().unwrap();
                device.broadcast_addr
            }
        }
    } else {
        frame::MacAddr::empty()
    };
    let interface = interface.0.lock().unwrap();
    interface.device.tx(ethernet::Type::Ip, data, mac_addr)
}
