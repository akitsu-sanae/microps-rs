use crate::{buffer::Buffer, packet};
use std::error::Error;

pub struct Packet {
    pub src_port: u16,
    pub dst_port: u16,
    pub len: u16,
    pub sum: u16,
    pub payload: Buffer,
}

#[cfg(debug_assertions)]
impl Packet {
    pub fn dump(&self) {
        eprintln!("src port: {}", self.src_port);
        eprintln!("dst port: {}", self.dst_port);
        eprintln!("len: {}", self.len);
        eprintln!("sum: {}", self.sum);
        eprintln!("{}", self.payload);
    }
}

impl packet::Packet<Packet> for Packet {
    fn from_buffer(mut buf: Buffer) -> Result<Self, Box<dyn Error>> {
        let src_port = buf.pop_u16("src port")?;
        let dst_port = buf.pop_u16("dst port")?;
        let len = buf.pop_u16("len")?;
        let sum = buf.pop_u16("sum")?;
        Ok(Packet {
            src_port: src_port,
            dst_port: dst_port,
            len: len,
            sum: sum,
            payload: buf,
        })
    }

    fn to_buffer(self) -> Buffer {
        let mut buf = Buffer::empty();
        buf.push_u16(self.src_port);
        buf.push_u16(self.dst_port);
        buf.push_u16(self.len);
        buf.push_u16(self.sum);
        buf.append(self.payload);
        buf
    }
}
