use crate::{arp::*, buffer::Buffer, ethernet, ip, packet, util::RuntimeError};

#[derive(Debug)]
pub struct Frame {
    pub op: Op,
    pub src_mac_addr: ethernet::MacAddr,
    pub src_ip_addr: ip::Addr,
    pub dst_mac_addr: ethernet::MacAddr,
    pub dst_ip_addr: ip::Addr,
}

const FRAME_SIZE: usize = 52;

impl Frame {
    pub fn dump(&self) {
        eprintln!("op: {}", self.op);
        eprintln!("src mac addr: {}", self.src_mac_addr);
        eprintln!("src ip addr: {}", self.src_ip_addr);
        eprintln!("dst mac addr: {}", self.dst_mac_addr);
        eprintln!("dst ip addr: {}", self.dst_ip_addr);
    }
}

impl packet::Packet<Frame> for Frame {
    fn from_buffer(mut buffer: Buffer) -> Result<Self, Box<dyn Error>> {
        let hardware_type = buffer.pop_u16("hardware type")?;
        if hardware_type != HARDWARE_TYPE_ETHERNET {
            return Err(RuntimeError::new(format!(
                "hardware type must be {}, but {}",
                HARDWARE_TYPE_ETHERNET, hardware_type
            )));
        }

        let protocol = buffer.pop_u16("protocol type")?;
        if protocol != ethernet::Type::Ip as u16 {
            return Err(RuntimeError::new(format!(
                "protocol type must be {}, but {}",
                ethernet::Type::Ip as u16,
                protocol
            )));
        }

        let hardware_address_len = buffer.pop_u8("hardware address length")?;
        if hardware_address_len as usize != ethernet::ADDR_LEN {
            return Err(RuntimeError::new(format!(
                "hardware address length must be {}, but {}",
                ethernet::ADDR_LEN,
                hardware_address_len
            )));
        }

        let ip_address_len = buffer.pop_u8("ip address length")?;
        if ip_address_len as usize != ip::ADDR_LEN {
            return Err(RuntimeError::new(format!(
                "ip address length must be {}, but {}",
                ip::ADDR_LEN,
                ip_address_len
            )));
        }

        let op: u16 = buffer.pop_u16("operation")?;
        let op = if op == Op::Request as u16 {
            Op::Request
        } else if op == Op::Reply as u16 {
            Op::Reply
        } else {
            return Err(RuntimeError::new(format!("invalid operation: {}", op)));
        };
        let src_mac_addr = buffer.pop_mac_addr("src mac address")?;
        let src_ip_addr = buffer.pop_ip_addr("src ip address")?;
        let dst_mac_addr = buffer.pop_mac_addr("dst mac address")?;
        let dst_ip_addr = buffer.pop_ip_addr("dst ip address")?;

        Ok(Frame {
            op: op,
            src_mac_addr: src_mac_addr,
            src_ip_addr: src_ip_addr,
            dst_mac_addr: dst_mac_addr,
            dst_ip_addr: dst_ip_addr,
        })
    }
    fn to_buffer(self) -> Buffer {
        let mut buffer = Buffer::new(FRAME_SIZE);
        buffer.push_u16(HARDWARE_TYPE_ETHERNET);
        buffer.push_u16(ethernet::Type::Ip as u16);
        buffer.push_u8(ethernet::ADDR_LEN as u8);
        buffer.push_u8(ip::ADDR_LEN as u8);
        buffer.push_u16(self.op as u16);
        buffer.push_mac_addr(self.src_mac_addr);
        buffer.push_ip_addr(self.src_ip_addr);
        buffer.push_mac_addr(self.dst_mac_addr);
        buffer.push_ip_addr(self.dst_ip_addr);
        buffer
    }
}
