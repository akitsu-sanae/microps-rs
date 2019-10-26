use crate::frame::{MacAddr, Bytes};
use std::error::Error;
use std::sync::{Arc, Mutex};

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

use std::fmt::Debug;
pub trait RawDevice : Debug {
    fn close(&mut self) -> Result<(), Box<dyn Error>>;
    fn rx(&mut self, callback: Box<dyn FnOnce(Bytes)>, timeout: i32);
    fn tx(&mut self, buf: &Vec<u8>) -> isize;

    fn type_(&self) -> Type;
    fn name(&self) -> &String;
    fn addr(&self) -> Result<MacAddr, Box<dyn Error>>;
}

fn detect_type(name: &str) -> Type {
    if name.starts_with("tap") {
        Type::Tap
    } else {
        DEFAULT_TYPE
    }
}

pub fn open(mut type_: Type, name: &str) -> Arc<Mutex<dyn RawDevice + Send>> {
    if type_ == Type::Auto {
        type_ = detect_type(name);
    }
    match type_ {
        Type::Auto => unreachable!(),
        Type::Tap => tap::Device::open(name).unwrap(),
        Type::Socket => socket::Device::open(name).unwrap(),
        // Type::Bpf => unimplemented!(),
    }
}
