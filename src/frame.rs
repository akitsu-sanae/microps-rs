use arrayvec::ArrayVec;
use std::collections::VecDeque;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct Bytes(VecDeque<u8>);

const MAC_ADDR_LEN: usize = 6;
const IPv4_ADDR_LEN: usize = 4;
const IPv6_ADDR_LEN: usize = 16;

pub struct MacAddr([u8; MAC_ADDR_LEN]);
pub struct Ipv4Addr([u8; IPv4_ADDR_LEN]);
pub struct Ipv6Addr([u8; IPv6_ADDR_LEN]);

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
        if IPv4_ADDR_LEN <= self.0.len() {
            let arr_vec: ArrayVec<[_; IPv4_ADDR_LEN]> =
                self.0.split_off(IPv4_ADDR_LEN).into_iter().collect();
            Some(Ipv4Addr(arr_vec.into_inner().unwrap()))
        } else {
            None
        }
    }
    pub fn pop_ipv6_addr(&mut self) -> Option<Ipv6Addr> {
        if IPv6_ADDR_LEN <= self.0.len() {
            let arr_vec: ArrayVec<[_; IPv6_ADDR_LEN]> =
                self.0.split_off(IPv6_ADDR_LEN).into_iter().collect();
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
