use crate::ethernet::ADDR_LEN;
use libc::{c_char, c_ulong};
use std::any::Any;
use std::error::Error;

pub mod socket;
pub mod tap;
// pub mod bpf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Auto,
    Tap,
    Socket,
    // Bpf,
}

// assume as HAVE_PF_PACKET
const DEFAULT_TYPE: Type = Type::Socket;

pub struct RawDeviceOps {
    pub open: fn(&RawDevice) -> Result<(), Box<dyn Error>>,
    pub close: fn(&RawDevice) -> Result<(), Box<dyn Error>>,
    pub rx: fn(&RawDevice, fn(&Vec<u8>, usize, &Vec<u8>), &Vec<u8>, i32),
    pub tx: fn(&RawDevice, buf: &Vec<u8>, len: usize) -> isize,
    pub addr: fn(&RawDevice, dst: &[u8; ADDR_LEN], usize) -> Result<(), Box<dyn Error>>,
}

pub struct RawDevice {
    pub type_: Type,
    pub name: String,
    pub ops: RawDeviceOps,
    data: Box<dyn Any>,
}

pub fn alloc(mut type_: Type, name: &str) -> RawDevice {
    if type_ == Type::Auto {
        type_ = match name {
            "tap" => Type::Tap,
            _ => DEFAULT_TYPE,
        }
    }
    let ops = match type_ {
        Type::Auto => unreachable!(),
        Type::Tap => unimplemented!(),
        Type::Socket => unimplemented!(),
        // Type::Bpf => unimplemented!(),
    };
    RawDevice {
        type_: type_,
        name: name.to_string(),
        ops: ops,
        data: Box::new(()),
    }
}

pub const TUNSETIFF: c_ulong = 1074025674;

#[derive(Debug)]
#[repr(C)]
pub struct ifreq {
    pub ifname: [c_char; 16],
    pub ifr_flags: i32,
    pub padding: [u8; 12],
}

impl Default for ifreq {
    fn default() -> ifreq {
        unsafe { std::mem::zeroed() }
    }
}
