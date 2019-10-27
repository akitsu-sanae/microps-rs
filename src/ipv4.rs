use std::error::Error;
use std::sync::{Arc, Mutex};

use crate::{ethernet, frame, interface, util};

pub const VERSION: usize = 4;

#[repr(u8)]
pub enum Protocol {
    Icmp = 0x01,
    Tcp = 0x06,
    Udp = 0x11,
    Raw = 0xff,
}

impl Protocol {
    pub fn from_u8(n: u8) -> Option<Protocol> {
        if n == Protocol::Icmp as u8 {
            Some(Protocol::Icmp)
        } else if n == Protocol::Tcp as u8 {
            Some(Protocol::Tcp)
        } else if n == Protocol::Udp as u8 {
            Some(Protocol::Udp)
        } else if n == Protocol::Raw as u8 {
            Some(Protocol::Raw)
        } else {
            None
        }
    }
}

use std::fmt;
impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Protocol::Icmp => "ICMP",
                Protocol::Tcp => "TCP",
                Protocol::Udp => "UDP",
                Protocol::Raw => "Raw",
            }
        )
    }
}

pub const HEADER_MIN_SIZE: usize = 20;
pub const HEADER_MAX_SIZE: usize = 60;
pub const PAYLOAD_MAX_SIZE: usize = 65535 - HEADER_MIN_SIZE;

pub struct Frame {
    pub version_header_length: u8,
    pub type_of_service: u8,
    pub len: u16,
    pub id: u16,
    pub offset: u16,
    pub time_to_live: u8,
    pub protocol: Protocol,
    pub checksum: u16,
    pub src: frame::Ipv4Addr,
    pub dst: frame::Ipv4Addr,
    pub options: Vec<u8>,
    pub payload: frame::Bytes,
}

impl frame::Frame for Frame {
    fn from_bytes(mut bytes: frame::Bytes) -> Result<Box<Self>, Box<dyn Error>> {
        let version_header_length = bytes.pop_u8("vhl")?;
        let type_of_service = bytes.pop_u8("tos")?;
        let len = bytes.pop_u16("length")?;
        let id = bytes.pop_u16("id")?;
        let offset = bytes.pop_u16("flags and fragment offset")?;
        let time_to_live = bytes.pop_u8("ttl")?;
        let protocol = bytes.pop_u8("protocol")?;
        let protocol = Protocol::from_u8(protocol).ok_or(util::RuntimeError::new(format!(
            "{} can not be Protocol Family",
            protocol
        )))?;
        let checksum = bytes.pop_u16("checksum")?;
        let src = bytes.pop_ipv4_addr("src")?;
        let dst = bytes.pop_ipv4_addr("dst")?;
        let options = bytes.pop_bytes(len as usize * 4 - 20, "options")?;
        let payload = bytes;

        Ok(Box::new(Frame {
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

// temporal definition for compiling
#[derive(Debug)]
pub struct InterfaceImpl {
    pub device: ethernet::Device,
    pub unicast: frame::Ipv4Addr,
    pub netmask: frame::Ipv4Addr,
    pub gateway: frame::Ipv4Addr,
}

#[derive(Debug, Clone)]
pub struct Interface(pub Arc<Mutex<InterfaceImpl>>);

impl Interface {
    pub fn new(inner: InterfaceImpl) -> Interface {
        Interface(Arc::new(Mutex::new(inner)))
    }
    pub fn family(&self) -> interface::Family {
        interface::Family::Ipv4
    }
    /*
    fn tx(&mut self, packet: frame::Bytes, dst: &Option<frame::Ipv4Addr>) -> Result<(), Box<dyn Error>> {
        let mac_addr = if self.device().inner().flags & ethernet::Flag::NOARP {
            match dst {
                Some(dst) => arp::resolve(self, dst, packet)?,
                None => self.device().inner().broadcast.clone()
            }
        } else {
            frame::MacAddr::empty()
        };
        self.device.tx(ethernet::Type::Ip, packet, mac_addr)
    } */
}

/* pub struct Fragment {
    pub src: frame::Ipv4Addr,
    pub dst: frame::Ipv4Addr,
    pub id: u16,
    pub protocol: Protocol,
    pub len: u16,
    pub data: [u8; 65535],
    pub mask: [u8; 2048],
    pub timestamp: DateTime<UTC>,
} */
