use std::error::Error;

use crate::net;

pub const ADDR_LEN: u16 = 6;
pub const ADDR_STR_LEN: usize = 18;

pub const HDR_SIZE: u16 = 14;
pub const TRL_SIZE: u16 = 4;
pub const FRAME_SIZE_MIN: u16 = 64;
pub const FRAME_SIZE_MAX: u16 = 1518;
pub const PAYLOAD_SIZE_MIN: u16 = FRAME_SIZE_MIN - (HDR_SIZE + TRL_SIZE);
pub const PAYLOAD_SIZE_MAX: u16 = FRAME_SIZE_MAX - (HDR_SIZE + TRL_SIZE);

fn open(dev: &net::Device, opt: i32) -> Result<(), Box<dyn Error>> {
    unimplemented!()
}

fn close(dev: &net::Device) -> Result<(), Box<dyn Error>> {
    unimplemented!()
}

fn run(dev: &net::Device) -> Result<(), Box<dyn Error>> {
    unimplemented!()
}

fn stop(dev: &net::Device) -> Result<(), Box<dyn Error>> {
    unimplemented!()
}

fn tx(
    dev: &net::Device,
    type_: u16,
    payload: &Vec<u8>,
    plen: usize,
    dst: &Vec<u8>,
) -> Result<(), Box<dyn Error>> {
    unimplemented!()
}

const ETHERNET_OPS: net::DeviceOps = net::DeviceOps {
    open: open,
    close: close,
    run: run,
    stop: stop,
    tx: tx,
};

const ETHERNET_DEF: net::DeviceDriver = net::DeviceDriver {
    type_: net::DeviceType::Ethernet,
    mtu: PAYLOAD_SIZE_MAX,
    flags: net::DeviceFlags::BROADCAST,
    hlen: HDR_SIZE,
    alen: ADDR_LEN,
    ops: ETHERNET_OPS,
};

pub fn init(net: &mut net::Net) -> Result<(), Box<dyn Error>> {
    net.regist_driver(ETHERNET_DEF)
}
