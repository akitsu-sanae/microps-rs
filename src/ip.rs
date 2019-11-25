use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use crate::{ethernet, frame, util, protocol::{Protocol, ProtocolType}, ip};

mod route;
mod fragment;

pub const VERSION: usize = 4;

pub const HEADER_MIN_SIZE: usize = 20;
pub const HEADER_MAX_SIZE: usize = 60;
pub const PAYLOAD_MAX_SIZE: usize = 65535 - HEADER_MIN_SIZE;

pub const ADDR_LEN: usize = 4;
const ADDR_BROADCAST : frame::IpAddr = frame::IpAddr([255; ADDR_LEN]);

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
    fn tx(&mut self, _protocol: ProtocolType, _packet: frame::Bytes, _dst: &Option<frame::IpAddr>) -> Result<(), Box<dyn Error>> {
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

fn forward_process(_dgram: &mut Dgram, _interface: &ip::Interface) -> Result<(), Box<dyn Error>> {
    unimplemented!()
}

fn fragment_process(_dgram: &Dgram) -> Result<fragment::Fragment, Box<dyn Error>> {
    unimplemented!()
}

pub fn rx(dgram: frame::Bytes, device: &ethernet::Device) -> Result<Option<JoinHandle<()>>, Box<dyn Error>> {
    use frame::Frame;
    let mut dgram = Dgram::from_bytes(dgram)?;
    let device = device.0.lock().unwrap();
    let interface = device.interface.as_ref().ok_or(util::RuntimeError::new(format!("device `{}` has not ip interface.", device.name)))?;
    let (unicast, broadcast) = {
        let interface = interface.0.lock().unwrap();
        (interface.unicast.clone(), interface.broadcast.clone())
    };
    if dgram.dst != unicast && dgram.dst != broadcast && dgram.dst != ADDR_BROADCAST {
        /* forward to other host */
        if IS_FORWARDING.load(Ordering::SeqCst) {
            forward_process(&mut dgram, interface);
        }
        return Ok(None);
    }
    dgram.dump();

    let payload = if dgram.offset & 0x2000 != 0 || dgram.offset & 0x1ff != 0 {
        let fragment = fragment_process(&dgram)?;
        fragment.data
    } else {
        dgram.payload
    };
    let protocols = PROTOCOLS.lock().unwrap();
    for protocol in protocols.iter() {
        if protocol.type_() == dgram.protocol {
            protocol.handler(payload, dgram.src, dgram.dst, interface)?;
            return Ok(None);
        }
    }
    Err(util::RuntimeError::new(format!("no suitable protocol")))
}

