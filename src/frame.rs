use crate::util;
use libc;
use arrayvec::ArrayVec;
use std::collections::VecDeque;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Bytes(pub VecDeque<u8>);

impl fmt::Display for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "-----+------------------------------------------------+------------------+\n")?;
        for i in 0..self.0.len()/16+1 {
            let offset = i * 16;
            write!(f, "{:>04} |", offset)?;
            for i in 0..16 {
                if offset+i < self.0.len() {
                    write!(f, "{:>02X} ", self.0[offset+i])?;
                } else {
                    write!(f, "   ")?;
                }
            }
            write!(f, "| ")?;
            for i in 0..16 {
                if offset+i < self.0.len() {
                    let c = self.0[offset+i] as char;
                    if c.is_ascii() && unsafe { libc::isprint(c as i32) != 0} {
                        write!(f, "{}", self.0[offset+i] as char)?;
                    } else {
                        write!(f, ".")?;
                    }
                } else {
                    write!(f, " ")?;
                }
            }
            write!(f, "\n")?;
        }
        write!(f, "-----+------------------------------------------------+------------------+\n")
    }
}

pub const MAC_ADDR_LEN: usize = 6;
pub const IPV4_ADDR_LEN: usize = 4;
pub const IPV6_ADDR_LEN: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacAddr(pub [u8; MAC_ADDR_LEN]);

impl MacAddr {
    pub fn from_str(str: &String) -> Result<Self, Box<dyn Error>> {
        str.split(':')
            .map(|n| u8::from_str_radix(n, 16))
            .collect::<Result<ArrayVec<[_; 6]>, _>>()
            .map(|arr| Self(arr.into_inner().unwrap()))
            .map_err(|err| Box::new(util::RuntimeError::new(format!("{}", err))) as Box<dyn Error>)
    }
}

impl fmt::Display for MacAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:X?}:{:X?}:{:X?}:{:X?}:{:X?}:{:X?}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ipv4Addr(pub [u8; IPV4_ADDR_LEN]);

impl Ipv4Addr {
    pub fn empty() -> Self {
        Ipv4Addr([0; IPV4_ADDR_LEN])
    }
    pub fn full() -> Self {
        Ipv4Addr([0xff; IPV4_ADDR_LEN])
    }

    pub fn from_str(str: String) -> Result<Self, Box<dyn Error>> {
        str.split(':')
            .map(|n| u8::from_str_radix(n, 10))
            .collect::<Result<ArrayVec<[_; 4]>, _>>()
            .map(|arr| Self(arr.into_inner().unwrap()))
            .map_err(|err| Box::new(util::RuntimeError::new(format!("{}", err))) as Box<dyn Error>)
    }
}

impl fmt::Display for Ipv4Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:#X}.{:#X}.{:#X}.{:#X}",
            self.0[0], self.0[1], self.0[2], self.0[3]
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ipv6Addr(pub [u8; IPV6_ADDR_LEN]);

impl Bytes {
    pub fn new(max_len: usize) -> Self {
        Bytes(VecDeque::with_capacity(max_len))
    }
    pub fn from_vec(vec: Vec<u8>) -> Self {
        Bytes(VecDeque::from(vec))
    }
    pub fn push_mac_addr(&mut self, addr: MacAddr) {
        self.0.append(&mut addr.0.into_iter().cloned().collect())
    }
    pub fn push_ipv4_addr(&mut self, addr: Ipv4Addr) {
        self.0.append(&mut addr.0.into_iter().cloned().collect())
    }
    pub fn push_ipv6_addr(&mut self, addr: Ipv6Addr) {
        self.0.append(&mut addr.0.into_iter().cloned().collect())
    }
    pub fn push_u8(&mut self, n: u8) {
        self.0.push_back(n);
    }
    pub fn push_u16(&mut self, n: u16) {
        self.0
            .append(&mut n.to_be_bytes().into_iter().cloned().collect());
    }
    pub fn push_u32(&mut self, n: u32) {
        self.0
            .append(&mut n.to_be_bytes().into_iter().cloned().collect());
    }

    pub fn pop_mac_addr(&mut self, label: &str) -> Result<MacAddr, Box<dyn Error>> {
        if MAC_ADDR_LEN <= self.0.len() {
            let buf = self.0.split_off(MAC_ADDR_LEN);
            let buf = ::std::mem::replace(&mut self.0, buf);
            let arr_vec: ArrayVec<[_; MAC_ADDR_LEN]> = buf.into_iter().collect();
            Ok(MacAddr(arr_vec.into_inner().unwrap()))
        } else {
            Err(Box::new(util::RuntimeError::new(format!("cannot pop {} from {:?}", label, self.0))))
        }
    }
    pub fn pop_ipv4_addr(&mut self, label: &str) -> Result<Ipv4Addr, Box<dyn Error>> {
        if IPV4_ADDR_LEN <= self.0.len() {
            let buf = self.0.split_off(IPV4_ADDR_LEN);
            let buf = ::std::mem::replace(&mut self.0, buf);
            let arr_vec: ArrayVec<[_; IPV4_ADDR_LEN]> = buf.into_iter().collect();
            Ok(Ipv4Addr(arr_vec.into_inner().unwrap()))
        } else {
            Err(Box::new(util::RuntimeError::new(format!("cannot pop {} from {:?}", label, self.0))))
        }
    }
    pub fn pop_ipv6_addr(&mut self, label: &str) -> Result<Ipv6Addr, Box<dyn Error>> {
        if IPV6_ADDR_LEN <= self.0.len() {
            let buf = self.0.split_off(IPV6_ADDR_LEN);
            let buf = ::std::mem::replace(&mut self.0, buf);
            let arr_vec: ArrayVec<[_; IPV6_ADDR_LEN]> = buf.into_iter().collect();
            Ok(Ipv6Addr(arr_vec.into_inner().unwrap()))
        } else {
            Err(Box::new(util::RuntimeError::new(format!("cannot pop {} from {:?}", label, self.0))))
        }
    }
    pub fn pop_u8(&mut self, label: &str) -> Result<u8, Box<dyn Error>> {
        self.0.pop_front().ok_or(Box::new(util::RuntimeError::new(format!("cannot pop {} from {:?}", label, self.0))))
    }

    pub fn pop_u16(&mut self, label: &str) -> Result<u16, Box<dyn Error>> {
        if 2 <= self.0.len() {
            let buf = self.0.split_off(2);
            let buf = ::std::mem::replace(&mut self.0, buf);
            let arr_vec: ArrayVec<[_; 2]> = buf.into_iter().collect();
            Ok(u16::from_be_bytes(arr_vec.into_inner().unwrap()))
        } else {
            Err(Box::new(util::RuntimeError::new(format!("cannot pop {} from {:?}", label, self.0))))
        }
    }
    pub fn pop_bytes(&mut self, len: usize, label: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        if len <= self.0.len() {
            let buf = self.0.split_off(len);
            let buf = ::std::mem::replace(&mut self.0, buf);
            Ok(buf.into_iter().collect())
        } else {
            Err(Box::new(util::RuntimeError::new(format!("cannot pop {} from {:?}", label, self.0))))
        }
    }
}

pub trait Frame {
    fn from_bytes(bytes: Bytes) -> Result<Box<Self>, Box<dyn Error>>;
    fn to_bytes(self) -> Bytes;
}
