use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use crate::{
    ethernet, frame, icmp, ip,
    protocol::{Protocol, ProtocolType},
    util,
};

mod fragment;
mod route;

pub const VERSION: usize = 4;

pub const HEADER_MIN_SIZE: usize = 20;
pub const HEADER_MAX_SIZE: usize = 60;
pub const PAYLOAD_MAX_SIZE: usize = 65535 - HEADER_MIN_SIZE;

pub const ADDR_LEN: usize = 4;
const ADDR_BROADCAST: frame::IpAddr = frame::IpAddr([255; ADDR_LEN]);

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
    pub options: Vec<u8>,
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
        eprintln!("options: {:?}", self.options); // TODO
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
        let options = bytes.pop_bytes(len as usize * 4 - 20, "options")?;
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
            options: options,
            payload: payload,
        }))
    }

    fn to_bytes(self) -> frame::Bytes {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct InterfaceImpl {
    pub device: ethernet::Device,
    pub unicast: frame::IpAddr,
    pub netmask: frame::IpAddr,
    pub network: frame::IpAddr,
    pub broadcast: frame::IpAddr,
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
        _protocol: ProtocolType,
        _packet: frame::Bytes,
        _dst: &frame::IpAddr,
    ) -> Result<(), Box<dyn Error>> {
        unimplemented!()
    }
}

use std::sync::atomic::{AtomicBool, Ordering};
lazy_static! {
    static ref IS_FORWARDING: AtomicBool = AtomicBool::new(false);
    static ref PROTOCOLS: Mutex<Vec<Arc<dyn Protocol + Send + Sync>>> = Mutex::new(vec![]);
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
    let route = match route::lookup(interface, dgram.dst) {
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
        &dgram,
        &match route.nexthop {
            Some(next) => next,
            None => dgram.src,
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
        (interface.unicast.clone(), interface.broadcast.clone())
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
    _interface: &Interface,
    _dgram: &Dgram,
    _dst: &frame::IpAddr,
) -> Result<(), Box<dyn Error>> {
    unimplemented!()
}
