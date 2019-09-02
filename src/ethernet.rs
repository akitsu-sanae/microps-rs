use std::error::Error;
use std::thread::{self, ThreadId};

use crate::{frame, net, raw, util};

pub const ADDR_LEN: usize = 6;
pub const ADDR_STR_LEN: usize = 18;

pub const HDR_SIZE: usize = 14;
pub const TRL_SIZE: usize = 4;
pub const FRAME_SIZE_MIN: usize = 64;
pub const FRAME_SIZE_MAX: usize = 1518;
pub const PAYLOAD_SIZE_MIN: usize = FRAME_SIZE_MIN - (HDR_SIZE + TRL_SIZE);
pub const PAYLOAD_SIZE_MAX: usize = FRAME_SIZE_MAX - (HDR_SIZE + TRL_SIZE);

const ADDR_ANY: [u8; ADDR_LEN] = [0; ADDR_LEN];
const ADDR_BROADCAST: [u8; ADDR_LEN] = [255; ADDR_LEN];

#[repr(u16)]
pub enum Type {
    Arp = 0x0806,
    Ipv4 = 0x0800,
    Ipv6 = 0x86DD,
}

impl Type {
    pub fn from_u16(n: u16) -> Option<Type> {
        if n == Type::Arp as u16 {
            Some(Type::Arp)
        } else if n == Type::Ipv4 as u16 {
            Some(Type::Ipv4)
        } else if n == Type::Ipv6 as u16 {
            Some(Type::Ipv6)
        } else {
            None
        }
    }
}

pub struct Frame {
    pub dst_addr: frame::MacAddr,
    pub src_addr: frame::MacAddr,
    pub type_: Type,
}

impl frame::Frame for Frame {
    fn from_bytes(mut bytes: frame::Bytes) -> Result<Box<Self>, Box<dyn Error>> {
        let mk_err = |name: &str, bytes: &frame::Bytes| {
            Box::new(util::RuntimeError::new(format!(
                "cannot pop {} from {:?}",
                name, bytes
            )))
        };

        let dst_addr = bytes.pop_mac_addr().ok_or(mk_err("dst addr", &bytes))?;
        let src_addr = bytes.pop_mac_addr().ok_or(mk_err("src addr", &bytes))?;
        let n = bytes.pop_u16().ok_or(mk_err("type", &bytes))?;
        let type_ = Type::from_u16(n).ok_or(Box::new(util::RuntimeError::new(format!(
            "{} can not be EthernetType",
            n
        ))))?;
        Ok(Box::new(Frame {
            dst_addr: dst_addr,
            src_addr: src_addr,
            type_: type_,
        }))
    }
    fn to_bytes(self) -> frame::Bytes {
        let mut bytes = frame::Bytes::new(FRAME_SIZE_MAX);
        bytes.push_mac_addr(self.dst_addr);
        bytes.push_mac_addr(self.src_addr);
        bytes.push_u16(self.type_ as u16);
        bytes
    }
}

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

fn close(_dev: &net::Device) -> Result<(), Box<dyn Error>> {
    unimplemented!()
}

fn run(_dev: &net::Device) -> Result<(), Box<dyn Error>> {
    unimplemented!()
}

fn stop(_dev: &net::Device) -> Result<(), Box<dyn Error>> {
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
