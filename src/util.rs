use crate::frame;
use std::error::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    pub message: String,
}

impl RuntimeError {
    pub fn new(message: String) -> Box<dyn Error> {
        Box::new(RuntimeError { message: message })
    }
}

use std::fmt;
impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "runtime error : {}", self.message)
    }
}

impl Error for RuntimeError {
    fn description(&self) -> &str {
        self.message.as_str()
    }
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

pub fn htons(n: u16) -> u16 {
    n.to_be()
}
pub fn ntohs(n: u16) -> u16 {
    u16::from_be(n)
}

pub fn hexdump(data: &Vec<u8>) {
    hexdump::hexdump(data.as_slice());
}

pub fn calc_checksum(mut data: frame::Bytes, init: u32) -> u16 {
    let mut sum = init;
    while data.0.len() >= 2 {
        sum += data.pop_u16("u16").unwrap() as u32;
    }
    if !data.0.is_empty() {
        sum += data.pop_u8("u8").unwrap() as u32;
    }
    sum = (sum & 0xffff) + (sum >> 16);
    sum = (sum & 0xffff) + (sum >> 16);
    return !(sum as u16);
}
