use crate::ethernet;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    Ipv4 = 0x02,
    Ipv6 = 0x0a,
}

use std::fmt;
impl fmt::Display for Family {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Family::Ipv4 => "IPv4",
                Family::Ipv6 => "IPv6",
            }
        )
    }
}

use std::fmt::Debug;
pub trait Interface: Debug {
    fn family(&self) -> Family;
    fn device(&self) -> ethernet::Device;
}
