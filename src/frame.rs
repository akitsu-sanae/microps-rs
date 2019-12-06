use crate::util;
use arrayvec::ArrayVec;
use libc;
use std::collections::VecDeque;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Bytes(pub VecDeque<u8>);

impl fmt::Display for Bytes {
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

pub const MAC_ADDR_LEN: usize = 6;
pub const IP_ADDR_LEN: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacAddr(pub [u8; MAC_ADDR_LEN]);

impl MacAddr {
    pub fn empty() -> MacAddr {
        MacAddr([0; MAC_ADDR_LEN])
    }
    pub fn from_str(str: &String) -> Result<Self, Box<dyn Error>> {
        str.split(':')
            .map(|n| u8::from_str_radix(n, 16))
            .collect::<Result<ArrayVec<[_; 6]>, _>>()
            .map(|arr| Self(arr.into_inner().unwrap()))
            .or_else(|err| Err(util::RuntimeError::new(format!("{}", err))))
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct IpAddr(pub [u8; IP_ADDR_LEN]);

impl IpAddr {
    pub fn empty() -> Self {
        IpAddr([0; IP_ADDR_LEN])
    }
    pub fn full() -> Self {
        IpAddr([0xff; IP_ADDR_LEN])
    }

    pub fn from_str(str: &String) -> Result<Self, Box<dyn Error>> {
        str.split('.')
            .map(|n| u8::from_str_radix(n, 10))
            .collect::<Result<ArrayVec<[_; 4]>, _>>()
            .map(|arr| Self(arr.into_inner().unwrap()))
            .or_else(|err| Err(util::RuntimeError::new(format!("{}", err))))
    }

    pub fn apply_mask(&self, mask: &IpAddr) -> IpAddr {
        IpAddr([
            self.0[0] & mask.0[0],
            self.0[1] & mask.0[1],
            self.0[2] & mask.0[2],
            self.0[3] & mask.0[3],
        ])
    }
}

use std::ops::BitOr;
impl BitOr for IpAddr {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        IpAddr([
            self.0[0] | rhs.0[0],
            self.0[1] | rhs.0[1],
            self.0[2] | rhs.0[2],
            self.0[3] | rhs.0[3],
        ])
    }
}

use std::ops::Not;
impl Not for IpAddr {
    type Output = Self;
    fn not(self) -> Self::Output {
        IpAddr([!self.0[0], !self.0[1], !self.0[2], !self.0[3]])
    }
}

impl fmt::Display for IpAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}.{}", self.0[0], self.0[1], self.0[2], self.0[3])
    }
}

impl Bytes {
    pub fn empty() -> Self {
        Bytes(VecDeque::new())
    }
    pub fn new(max_len: usize) -> Self {
        Bytes(VecDeque::with_capacity(max_len))
    }
    pub fn head(&mut self, len: usize) -> Self {
        let head = self.0.split_off(len);
        let mut head_ = head.clone();
        head_.append(&mut self.0);
        ::std::mem::replace(&mut self.0, head_);
        Bytes(head)
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn from_vec(vec: Vec<u8>) -> Self {
        Bytes(VecDeque::from(vec))
    }
    pub fn to_vec(self) -> Vec<u8> {
        self.0.into_iter().collect()
    }
    pub fn push_mac_addr(&mut self, addr: MacAddr) {
        self.0.append(&mut addr.0.iter().cloned().collect())
    }
    pub fn push_ip_addr(&mut self, addr: IpAddr) {
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
    pub fn append(&mut self, mut buf: Bytes) {
        self.0.append(&mut buf.0)
    }
    pub fn write(&mut self, pos: usize, mut buf: Bytes) {
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

    pub fn pop_mac_addr(&mut self, label: &str) -> Result<MacAddr, Box<dyn Error>> {
        if MAC_ADDR_LEN <= self.0.len() {
            let buf = self.0.split_off(MAC_ADDR_LEN);
            let buf = ::std::mem::replace(&mut self.0, buf);
            let arr_vec: ArrayVec<[_; MAC_ADDR_LEN]> = buf.into_iter().collect();
            Ok(MacAddr(arr_vec.into_inner().unwrap()))
        } else {
            Err(util::RuntimeError::new(format!(
                "cannot pop {} from {:?}",
                label, self.0
            )))
        }
    }
    pub fn pop_ip_addr(&mut self, label: &str) -> Result<IpAddr, Box<dyn Error>> {
        if IP_ADDR_LEN <= self.0.len() {
            let buf = self.0.split_off(IP_ADDR_LEN);
            let buf = ::std::mem::replace(&mut self.0, buf);
            let arr_vec: ArrayVec<[_; IP_ADDR_LEN]> = buf.into_iter().collect();
            Ok(IpAddr(arr_vec.into_inner().unwrap()))
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

    pub fn pop_bytes(&mut self, len: usize, label: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        if len <= self.0.len() {
            let buf = self.0.split_off(len);
            let buf = ::std::mem::replace(&mut self.0, buf);
            Ok(buf.into_iter().collect())
        } else {
            Err(util::RuntimeError::new(format!(
                "cannot pop {} from {:?}",
                label, self.0
            )))
        }
    }
}

pub trait Frame {
    fn from_bytes(bytes: Bytes) -> Result<Box<Self>, Box<dyn Error>>;
    fn to_bytes(self) -> Bytes;
}
