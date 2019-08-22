use std::error::Error;

use crate::{net, raw, util};

pub const ADDR_LEN: u16 = 6;
pub const ADDR_STR_LEN: usize = 18;

pub const HDR_SIZE: u16 = 14;
pub const TRL_SIZE: u16 = 4;
pub const FRAME_SIZE_MIN: u16 = 64;
pub const FRAME_SIZE_MAX: u16 = 1518;
pub const PAYLOAD_SIZE_MIN: u16 = FRAME_SIZE_MIN - (HDR_SIZE + TRL_SIZE);
pub const PAYLOAD_SIZE_MAX: u16 = FRAME_SIZE_MAX - (HDR_SIZE + TRL_SIZE);

fn open(dev: &net::Device, opt: raw::Type) -> Result<(), Box<dyn Error>> {
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
    _dev: &net::Device,
    _type_: u16,
    _payload: &Vec<u8>,
    _plen: usize,
    _dst: &Vec<u8>,
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

pub fn addr_pton(p: &String) -> Result<[u8; 16], util::RuntimeError> {
    use arrayvec::ArrayVec;
    p.split(':')
        .map(|n| u8::from_str_radix(n, 16))
        .collect::<Result<ArrayVec<[_; 16]>, _>>()
        .map(|arr| arr.into_inner().unwrap())
        .map_err(|err| util::RuntimeError::new(format!("{}", err)))
}
