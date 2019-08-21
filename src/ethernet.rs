use std::error::Error;

use crate::net;

pub const ADDR_LEN: usize = 6;
pub const ADDR_STR_LEN: usize = 18;

pub const HDR_SIZE: usize = 14;
pub const TRL_SIZE: usize = 4;
pub const FRAME_SIZE_MIN: usize = 64;
pub const FRAME_SIZE_MAX: usize = 1518;
pub const PAYLOAD_SIZE_MIN: usize = FRAME_SIZE_MIN - (HDR_SIZE + TRL_SIZE);
pub const PAYLOAD_SIZE_MAX: usize = FRAME_SIZE_MAX - (HDR_SIZE + TRL_SIZE);

fn open(dev: &net::Netdev, opt: i32) -> Result<(), Box<dyn Error>> {
    unimplemented!()
}

fn close(dev: &net::Netdev) -> Result<(), Box<dyn Error>> {
    unimplemented!()
}

fn run(dev: &net::Netdev) -> Result<(), Box<dyn Error>> {
    unimplemented!()
}

fn stop(dev: &net::Netdev) -> Result<(), Box<dyn Error>> {
    unimplemented!()
}

fn tx(
    dev: &net::Netdev,
    type_: u16,
    payload: &Vec<u8>,
    plen: usize,
    dst: &Vec<u8>,
) -> Result<(), Box<dyn Error>> {
    unimplemented!()
}

const ETHERNET_OPS: net::NetdevOps = net::NetdevOps {
    open: open,
    close: close,
    run: run,
    stop: stop,
    tx: tx,
};

const ETHERNET_DEF: net::NetdevDef = net::NetdevDef {
    type_: net::Type::Ethernet,
    mtu: PAYLOAD_SIZE_MAX,
    flags: net::Flag::Broadcast,
    hlen: HDR_SIZE,
    alen: ADDR_LEN,
    ops: ETHERNET_OPS,
};

pub fn init() -> Result<(), Box<dyn Error>> {
    net::regist_driver(&ETHERNET_DEF)
}
