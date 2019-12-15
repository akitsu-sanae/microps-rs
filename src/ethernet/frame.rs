use crate::{buffer::Buffer, ethernet, packet::Packet, util::RuntimeError};
use std::error::Error;

pub struct Frame {
    pub dst_addr: ethernet::MacAddr,
    pub src_addr: ethernet::MacAddr,
    pub type_: ethernet::Type,
    pub payload: Buffer,
}

#[cfg(debug_assertions)]
impl Frame {
    pub fn dump(&self) {
        eprintln!("dst : {}", self.dst_addr);
        eprintln!("src : {}", self.src_addr);
        eprintln!("type: {}", self.type_);
        eprintln!("{}", self.payload);
    }
}

impl Packet<Frame> for Frame {
    fn from_buffer(mut buf: Buffer) -> Result<Self, Box<dyn Error>> {
        let dst_addr = buf.pop_mac_addr("dst addr")?;
        let src_addr = buf.pop_mac_addr("src addr")?;
        let n = buf.pop_u16("type")?;
        let type_ = ethernet::Type::from_u16(n)
            .ok_or(RuntimeError::new(format!("{} can not be EthernetType", n)))?;
        Ok(Frame {
            dst_addr: dst_addr,
            src_addr: src_addr,
            type_: type_,
            payload: buf,
        })
    }
    fn to_buffer(self) -> Buffer {
        let mut buf = Buffer::new(ethernet::FRAME_SIZE_MAX);
        buf.push_mac_addr(self.dst_addr);
        buf.push_mac_addr(self.src_addr);
        buf.push_u16(self.type_ as u16);
        buf.append(self.payload);
        buf
    }
}
