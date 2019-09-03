use bitflags::bitflags;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Ethernet,
    Slip,
}

bitflags! {
    pub struct DeviceFlags: u32 {
        const BROADCAST = 0x01;
        const MULTICAST = 0x02;
        const P2P       = 0x04;
        const LOOPBACK  = 0x08;
        const NOARP     = 0x10;
        const PROMISC   = 0x20;
        const RUNNING   = 0x40;
        const UP        = 0x80;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interface {
    Ipv4,
    Ipv6,
}
