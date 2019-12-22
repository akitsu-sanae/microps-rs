use crate::{buffer, ip, packet, protocol::ProtocolType, util};
use std::error::Error;

pub const HEADER_MIN_SIZE: usize = 20;
pub const HEADER_MAX_SIZE: usize = 60;
pub const PAYLOAD_MAX_SIZE: usize = 65535 - HEADER_MIN_SIZE;

pub const HEADER_LEN: u8 = 1 + 1 + 2 + 2 + 2 + 1 + 1 + 2 + 4 + 4 + 1;

#[derive(Debug, Clone)]
pub struct Dgram {
    pub version_header_length: u8,
    pub type_of_service: u8,
    pub len: u16,
    pub id: u16,
    pub offset: u16,
    pub time_to_live: u8,
    pub protocol: ProtocolType,
    pub checksum: u16,
    pub src: ip::Addr,
    pub dst: ip::Addr,
    pub payload: buffer::Buffer,
}

impl Dgram {
    pub fn dump(&self) {
        eprintln!("version, header length: {}", self.version_header_length);
        eprintln!("type of service: {}", self.type_of_service);
        eprintln!("len: {}", self.len);
        eprintln!("id: {}", self.id);
        eprintln!("offset: {}", self.offset);
        eprintln!("time_to_live: {}", self.time_to_live);
        eprintln!("protocol: {}", self.protocol);
        eprintln!("checksum: {}", self.checksum);
        eprintln!("src: {}", self.src);
        eprintln!("dst: {}", self.dst);
        eprintln!("payload: {}", self.payload);
    }
}

impl packet::Packet<Dgram> for Dgram {
    fn from_buffer(mut buf: buffer::Buffer) -> Result<Self, Box<dyn Error>> {
        let version_header_length = buf.pop_u8("vhl")?;
        let type_of_service = buf.pop_u8("tos")?;
        let len = buf.pop_u16("length")?;
        let id = buf.pop_u16("id")?;
        let offset = buf.pop_u16("flags and fragment offset")?;
        let time_to_live = buf.pop_u8("ttl")?;
        let protocol = buf.pop_u8("protocol")?;
        let protocol = ProtocolType::from_u8(protocol).ok_or(util::RuntimeError::new(format!(
            "{} can not be Protocol Family",
            protocol
        )))?;
        let checksum = buf.pop_u16("checksum")?;
        let src = buf.pop_ip_addr("src")?;
        let dst = buf.pop_ip_addr("dst")?;
        let payload = buf;

        Ok(Dgram {
            version_header_length: version_header_length,
            type_of_service: type_of_service,
            len: len,
            id: id,
            offset: offset,
            time_to_live: time_to_live,
            protocol: protocol,
            checksum: checksum,
            src: src,
            dst: dst,
            payload: payload,
        })
    }

    fn to_buffer(self) -> buffer::Buffer {
        let mut buf = buffer::Buffer::new(HEADER_MAX_SIZE);
        buf.push_u8(self.version_header_length);
        buf.push_u8(self.type_of_service);
        buf.push_u16(self.len);
        buf.push_u16(self.id);
        buf.push_u16(self.offset);
        buf.push_u8(self.time_to_live);
        buf.push_u8(self.protocol as u8);
        buf.push_u16(self.checksum);
        buf.push_ip_addr(self.src);
        buf.push_ip_addr(self.dst);
        buf.append(self.payload);

        buf
    }
}
