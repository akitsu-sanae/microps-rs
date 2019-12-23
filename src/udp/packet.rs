use crate::{buffer::Buffer, packet, util};
use std::error::Error;

pub struct Packet {
    pub src_port: u16,
    pub dst_port: u16,
    pub sum: u16,
    pub payload: Buffer,
}

const HEADER_LEN: usize = 8;

impl Packet {
    pub fn dump(&self) {
        eprintln!("src port: {}", self.src_port);
        eprintln!("dst port: {}", self.dst_port);
        eprintln!("sum: {}", self.sum);
        eprintln!("{}", self.payload);
    }

    pub fn write_checksum(buf: &mut Buffer, sum: u16) {
        buf.write_u16(4, sum);
    }
}

impl packet::Packet<Packet> for Packet {
    fn from_buffer(mut buf: Buffer) -> Result<Self, Box<dyn Error>> {
        let src_port = buf.pop_u16("src port")?;
        let dst_port = buf.pop_u16("dst port")?;
        let len = buf.pop_u16("len")?;
        let sum = buf.pop_u16("sum")?;
        if len as usize != HEADER_LEN + buf.0.len() {
            return Err(util::RuntimeError::new(format!("invalid len: {}", len)));
        }
        Ok(Packet {
            src_port: src_port,
            dst_port: dst_port,
            sum: sum,
            payload: buf,
        })
    }

    fn to_buffer(self) -> Buffer {
        let mut buf = Buffer::empty();
        buf.push_u16(self.src_port);
        buf.push_u16(self.dst_port);
        buf.push_u16((HEADER_LEN + self.payload.0.len()) as u16);
        buf.push_u16(self.sum);
        buf.append(self.payload);
        buf
    }
}
