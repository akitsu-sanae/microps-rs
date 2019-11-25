use crate::{frame, ip};
use std::error::Error;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolType {
    Icmp = 0x01,
    Tcp = 0x06,
    Udp = 0x11,
    Raw = 0xff,
}

impl ProtocolType {
    pub fn from_u8(n: u8) -> Option<ProtocolType> {
        if n == ProtocolType::Icmp as u8 {
            Some(ProtocolType::Icmp)
        } else if n == ProtocolType::Tcp as u8 {
            Some(ProtocolType::Tcp)
        } else if n == ProtocolType::Udp as u8 {
            Some(ProtocolType::Udp)
        } else if n == ProtocolType::Raw as u8 {
            Some(ProtocolType::Raw)
        } else {
            None
        }
    }
}

use std::fmt;
impl fmt::Display for ProtocolType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ProtocolType::Icmp => "ICMP",
                ProtocolType::Tcp => "TCP",
                ProtocolType::Udp => "UDP",
                ProtocolType::Raw => "Raw",
            }
        )
    }
}

pub trait Protocol {
    fn type_(&self) -> ProtocolType;
    fn handler(
        &self,
        payload: frame::Bytes,
        src: frame::IpAddr,
        dst: frame::IpAddr,
        interface: &ip::Interface,
    ) -> Result<(), Box<dyn Error>>;
}
