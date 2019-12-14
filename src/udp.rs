use std::error::Error;
use crate::{ip, buffer};

pub struct Socket {}

impl Socket {
    pub fn bind(&mut self, _peer_addr: Option<ip::Addr>, _peer_port: i32) -> Result<(), Box<dyn Error>> {
        unimplemented!()
    }

    pub fn recv_from(&mut self, _timeout: i32) -> Result<(ip::Addr, i32, buffer::Buffer), Box<dyn Error>> {
        unimplemented!()
    }

    pub fn send_to(&mut self, _buf: buffer::Buffer, _peer_addr: ip::Addr, _peer_port: i32) -> Result<(), Box<dyn Error>> {
        unimplemented!()
    }

    pub fn close(&self) -> Result<(), Box<dyn Error>> {
        unimplemented!()
    }
}

pub fn open() -> Result<Socket, Box<dyn Error>> {
    unimplemented!()
}



