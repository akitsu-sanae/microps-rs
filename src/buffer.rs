use crate::{ethernet, ip, util};
use arrayvec::ArrayVec;
use std::collections::VecDeque;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct Buffer(pub VecDeque<u8>);

use std::fmt;
impl fmt::Display for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "-----+------------------------------------------------+------------------+\n"
        )?;
        for i in 0..self.0.len() / 16 + 1 {
            let offset = i * 16;
            write!(f, "{:>04} |", offset)?;
            for i in 0..16 {
                if offset + i < self.0.len() {
                    write!(f, "{:>02X} ", self.0[offset + i])?;
                } else {
                    write!(f, "   ")?;
                }
            }
            write!(f, "| ")?;
            for i in 0..16 {
                if offset + i < self.0.len() {
                    let c = self.0[offset + i] as char;
                    if c.is_ascii() && unsafe { libc::isprint(c as i32) != 0 } {
                        write!(f, "{}", self.0[offset + i] as char)?;
                    } else {
                        write!(f, ".")?;
                    }
                } else {
                    write!(f, " ")?;
                }
            }
            write!(f, "\n")?;
        }
        write!(
            f,
            "-----+------------------------------------------------+------------------+\n"
        )
    }
}

impl Buffer {
    pub fn empty() -> Self {
        Buffer(VecDeque::new())
    }
    pub fn new(max_len: usize) -> Self {
        Buffer(VecDeque::with_capacity(max_len))
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn from_vec(vec: Vec<u8>) -> Self {
        Buffer(VecDeque::from(vec))
    }
    pub fn to_vec(self) -> Vec<u8> {
        self.0.into_iter().collect()
    }
    pub fn push_mac_addr(&mut self, addr: ethernet::MacAddr) {
        self.0.append(&mut addr.0.iter().cloned().collect())
    }
    pub fn push_ip_addr(&mut self, addr: ip::Addr) {
        self.0.append(&mut addr.0.iter().cloned().collect())
    }
    pub fn push_u8(&mut self, n: u8) {
        self.0.push_back(n);
    }
    pub fn push_u16(&mut self, n: u16) {
        self.0
            .append(&mut n.to_be_bytes().iter().cloned().collect());
    }
    pub fn push_u32(&mut self, n: u32) {
        self.0
            .append(&mut n.to_be_bytes().iter().cloned().collect());
    }
    pub fn append(&mut self, mut buf: Buffer) {
        self.0.append(&mut buf.0)
    }
    pub fn write(&mut self, pos: usize, mut buf: Buffer) {
        let mut after = self.0.split_off(pos);
        if buf.0.len() < after.len() {
            let mut after = after.split_off(buf.0.len());
            self.0.append(&mut buf.0);
            self.0.append(&mut after);
        } else {
            buf.0.split_off(after.len());
            self.0.append(&mut buf.0);
        }
    }
    pub fn write_u16(&mut self, pos: usize, v: u16) {
        self.0[pos] = v as u8;
        self.0[pos + 1] = (v >> 8) as u8;
    }
    pub fn pop_mac_addr(&mut self, label: &str) -> Result<ethernet::MacAddr, Box<dyn Error>> {
        if ethernet::ADDR_LEN <= self.0.len() {
            let buf = self.0.split_off(ethernet::ADDR_LEN);
            let buf = ::std::mem::replace(&mut self.0, buf);
            let arr_vec: ArrayVec<[_; ethernet::ADDR_LEN]> = buf.into_iter().collect();
            Ok(ethernet::MacAddr(arr_vec.into_inner().unwrap()))
        } else {
            Err(util::RuntimeError::new(format!(
                "cannot pop {} from {:?}",
                label, self.0
            )))
        }
    }
    pub fn pop_ip_addr(&mut self, label: &str) -> Result<ip::Addr, Box<dyn Error>> {
        if ip::ADDR_LEN <= self.0.len() {
            let buf = self.0.split_off(ip::ADDR_LEN);
            let buf = ::std::mem::replace(&mut self.0, buf);
            let arr_vec: ArrayVec<[_; ip::ADDR_LEN]> = buf.into_iter().collect();
            Ok(ip::Addr(arr_vec.into_inner().unwrap()))
        } else {
            Err(util::RuntimeError::new(format!(
                "cannot pop {} from {:?}",
                label, self.0
            )))
        }
    }
    pub fn pop_u8(&mut self, label: &str) -> Result<u8, Box<dyn Error>> {
        self.0.pop_front().ok_or(util::RuntimeError::new(format!(
            "cannot pop {} from {:?}",
            label, self.0
        )))
    }

    pub fn pop_u16(&mut self, label: &str) -> Result<u16, Box<dyn Error>> {
        if 2 <= self.0.len() {
            let buf = self.0.split_off(2);
            let buf = ::std::mem::replace(&mut self.0, buf);
            let arr_vec: ArrayVec<[_; 2]> = buf.into_iter().collect();
            Ok(u16::from_be_bytes(arr_vec.into_inner().unwrap()))
        } else {
            Err(util::RuntimeError::new(format!(
                "cannot pop {} from {:?}",
                label, self.0
            )))
        }
    }

    pub fn pop_u32(&mut self, label: &str) -> Result<u32, Box<dyn Error>> {
        if 4 <= self.0.len() {
            let buf = self.0.split_off(4);
            let buf = ::std::mem::replace(&mut self.0, buf);
            let arr_vec: ArrayVec<[_; 4]> = buf.into_iter().collect();
            Ok(u32::from_be_bytes(arr_vec.into_inner().unwrap()))
        } else {
            Err(util::RuntimeError::new(format!(
                "cannot pop {} from {:?}",
                label, self.0
            )))
        }
    }

    pub fn pop_buffer(&mut self, len: usize, label: &str) -> Result<Buffer, Box<dyn Error>> {
        if len <= self.0.len() {
            let buf = self.0.split_off(len);
            let buf = ::std::mem::replace(&mut self.0, buf);
            Ok(Buffer(buf))
        } else {
            Err(util::RuntimeError::new(format!(
                "cannot pop {} from {:?}",
                label, self.0
            )))
        }
    }
}
