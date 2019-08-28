use crate::ethernet::ADDR_LEN;
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

pub trait RawDevice {
    fn close(&mut self) -> Result<(), Box<dyn Error>>;
    fn rx(&mut self, callback: fn(&Vec<u8>, usize, &Vec<u8>), arg: &Vec<u8>, timeout: i32);
    fn tx(&mut self, buf: &Vec<u8>) -> isize;

    fn type_(&self) -> Type;
    fn name(&self) -> &String;
    fn addr(&self) -> Result<[u8; ADDR_LEN], Box<dyn Error>>;
}

pub fn open(mut type_: Type, name: &str) -> Box<dyn RawDevice> {
    if type_ == Type::Auto {
        type_ = match name {
            "tap" => Type::Tap,
            _ => DEFAULT_TYPE,
        }
    }
    match type_ {
        Type::Auto => unreachable!(),
        Type::Tap => tap::TapDevice::open(name).unwrap(),
        Type::Socket => socket::SocketDevice::open(name).unwrap(),
        // Type::Bpf => unimplemented!(),
    }
}

