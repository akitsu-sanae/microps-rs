use std::error::Error;
use std::thread::{self, ThreadId};

use crate::{net, raw, util};

pub const ADDR_LEN: usize = 6;
pub const ADDR_STR_LEN: usize = 18;

pub const HDR_SIZE: u16 = 14;
pub const TRL_SIZE: u16 = 4;
pub const FRAME_SIZE_MIN: u16 = 64;
pub const FRAME_SIZE_MAX: u16 = 1518;
pub const PAYLOAD_SIZE_MIN: u16 = FRAME_SIZE_MIN - (HDR_SIZE + TRL_SIZE);
pub const PAYLOAD_SIZE_MAX: u16 = FRAME_SIZE_MAX - (HDR_SIZE + TRL_SIZE);

const ADDR_ANY: [u8; ADDR_LEN] = [0; ADDR_LEN];
const ADDR_BROADCAST: [u8; ADDR_LEN] = [255; ADDR_LEN];

struct Data {
    // pub device: net::Device,
    // pub raw_device: raw::RawDevice,
    pub thread: ThreadId,
    pub terminate: bool,
}

fn open(device: &mut net::Device, opt: raw::Type) -> Result<(), Box<dyn Error>> {
    let raw_device = raw::open(opt, device.name.as_str());
    device.data = Box::new(Data {
        // device: device,
        // raw_device: &raw_device,
        thread: thread::current().id(),
        terminate: false,
    });
    if device.addr == ADDR_ANY {
        device.addr = raw_device.addr()?;
    }
    device.broadcast = ADDR_BROADCAST;
    Ok(())
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

pub fn addr_pton(p: &String) -> Result<[u8; ADDR_LEN], util::RuntimeError> {
    use arrayvec::ArrayVec;
    use std::convert::TryInto;
    p.split(':')
        .map(|n| u8::from_str_radix(n, ADDR_LEN.try_into().unwrap()))
        .collect::<Result<ArrayVec<[_; ADDR_LEN]>, _>>()
        .map(|arr| arr.into_inner().unwrap())
        .map_err(|err| util::RuntimeError::new(format!("{}", err)))
}
