use crate::ethernet::ADDR_LEN;
use std::any::Any;
use std::error::Error;

pub enum Type {
    Auto,
    Tap,
    Socket,
    Bpf,
}

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

pub fn alloc(type_: Type, name: &str) -> RawDevice {
    unimplemented!()
}
