use std::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Ethernet,
    Slip,
}

#[derive(Debug, Clone)]
pub enum Flag {
    Broadcast,
    Multicast,
    P2p,
    Loopback,
    Noarp,
    Promisc,
    Running,
    Up,
}

pub struct NetdevOps {
    pub open: fn(&Netdev, i32) -> Result<(), Box<dyn Error>>,
    pub close: fn(&Netdev) -> Result<(), Box<dyn Error>>,
    pub run: fn(&Netdev) -> Result<(), Box<dyn Error>>,
    pub stop: fn(&Netdev) -> Result<(), Box<dyn Error>>,
    pub tx: fn(&Netdev, u16, &Vec<u8>, usize, &Vec<u8>) -> Result<(), Box<dyn Error>>,
}

pub struct NetdevDef {
    pub type_: Type,
    pub mtu: usize,
    pub flags: Flag,
    pub hlen: usize,
    pub alen: usize,
    pub ops: NetdevOps,
}

#[derive(Debug, Clone)]
pub struct Netdev {}

pub fn regist_driver(def: &NetdevDef) -> Result<(), Box<dyn Error>> {
    unimplemented!()
}
