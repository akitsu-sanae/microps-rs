use crate::util;
use arrayvec::ArrayVec;
use std::collections::VecDeque;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Bytes(VecDeque<u8>);

const MAC_ADDR_LEN: usize = 6;
const IPV4_ADDR_LEN: usize = 4;
const IPV6_ADDR_LEN: usize = 16;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Ipv4Addr(pub [u8; IPV4_ADDR_LEN]);

impl Ipv4Addr {
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

#[derive(Debug, Clone)]
pub struct Ipv6Addr(pub [u8; IPV6_ADDR_LEN]);

impl Bytes {
    pub fn new(max_len: usize) -> Self {
        Bytes(VecDeque::with_capacity(max_len))
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

    pub fn pop_mac_addr(&mut self) -> Option<MacAddr> {
        if MAC_ADDR_LEN <= self.0.len() {
            let arr_vec: ArrayVec<[_; MAC_ADDR_LEN]> =
                self.0.split_off(MAC_ADDR_LEN).into_iter().collect();
            Some(MacAddr(arr_vec.into_inner().unwrap()))
        } else {
            None
        }
    }
    pub fn pop_ipv4_addr(&mut self) -> Option<Ipv4Addr> {
        if IPV4_ADDR_LEN <= self.0.len() {
            let arr_vec: ArrayVec<[_; IPV4_ADDR_LEN]> =
                self.0.split_off(IPV4_ADDR_LEN).into_iter().collect();
            Some(Ipv4Addr(arr_vec.into_inner().unwrap()))
        } else {
            None
        }
    }
    pub fn pop_ipv6_addr(&mut self) -> Option<Ipv6Addr> {
        if IPV6_ADDR_LEN <= self.0.len() {
            let arr_vec: ArrayVec<[_; IPV6_ADDR_LEN]> =
                self.0.split_off(IPV6_ADDR_LEN).into_iter().collect();
            Some(Ipv6Addr(arr_vec.into_inner().unwrap()))
        } else {
            None
        }
    }
    pub fn pop_u16(&mut self) -> Option<u16> {
        if 2 <= self.0.len() {
            let arr_vec: ArrayVec<[_; 2]> = self.0.split_off(2).into_iter().collect();
            Some(u16::from_be_bytes(arr_vec.into_inner().unwrap()))
        } else {
            None
        }
    }
    pub fn rest(self) -> Vec<u8> {
        self.0.into_iter().collect()
    }
}

pub trait Frame {
    fn from_bytes(bytes: Bytes) -> Result<Box<Self>, Box<dyn Error>>;
    fn to_bytes(self) -> Bytes;
}
