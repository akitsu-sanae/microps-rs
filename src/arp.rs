use std::error::Error;
use std::sync::{Arc, Mutex, Condvar};
use chrono::{Utc, DateTime};
use nix::errno::{errno, Errno};
use crate::{frame, ethernet, util, device, ip};

const HARDWARE_TYPE_ETHERNET: u16 = 0x0001;

#[repr(u16)]
pub enum Op {
    Request = 1,
    Reply = 2,
}

impl Op {
    pub fn from_u16(n: u16) -> Option<Op> {
        if n == Op::Request as u16 {
            Some(Op::Request)
        } else if n == Op::Reply as u16 {
            Some(Op::Reply)
        } else {
            None
        }
    }
}

use std::fmt;
impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter)-> fmt::Result {
        write!(f, "{}", match self {
            Op::Request => "Request",
            Op::Reply => "Reply",
        })
    }
}

pub struct Frame {
    pub op: Op,
    pub src_mac_addr: frame::MacAddr,
    pub src_ip_addr: frame::Ipv4Addr,
    pub dst_mac_addr: frame::MacAddr,
    pub dst_ip_addr: frame::Ipv4Addr,
}

const FRAME_SIZE : usize = 52;

impl Frame {
    pub fn dump(&self) {
        eprintln!("op: {}", self.op);
        eprintln!("src mac addr: {}", self.src_mac_addr);
        eprintln!("src ip addr: {}", self.src_ip_addr);
        eprintln!("dst mac addr: {}", self.dst_mac_addr);
        eprintln!("dst ip addr: {}", self.dst_ip_addr);
    }
}

impl frame::Frame for Frame {
    fn from_bytes(mut bytes: frame::Bytes) -> Result<Box<Self>, Box<dyn Error>> {
        let hardware_type = bytes.pop_u16("hardware type")?;
        if hardware_type != HARDWARE_TYPE_ETHERNET {
            return Err(Box::new(util::RuntimeError::new(format!("hardware type must be {}, but {}", HARDWARE_TYPE_ETHERNET, hardware_type))));
        }

        let protocol = bytes.pop_u16("protocol type")?;
        if protocol != ethernet::Type::Ipv4 as u16 {
            return Err(Box::new(util::RuntimeError::new(format!("protocol type must be {}, but {}", ethernet::Type::Ipv4 as u16, protocol))));
        }

        let hardware_address_len = bytes.pop_u8("hardware address length")?;
        if hardware_address_len as usize != frame::MAC_ADDR_LEN {
            return Err(Box::new(util::RuntimeError::new(format!("hardware address length must be {}, but {}", frame::MAC_ADDR_LEN, hardware_address_len))));
        }

        let ip_address_len = bytes.pop_u8("ip address length")?;
        if ip_address_len as usize != frame::IPV4_ADDR_LEN {
            return Err(Box::new(util::RuntimeError::new(format!("ip address length must be {}, but {}", frame::IPV4_ADDR_LEN, ip_address_len))));
        }

        let op: u16 = bytes.pop_u16("operation")?;
        let op = if op == Op::Request as u16 {
            Op::Request
        } else if op == Op::Reply as u16 {
            Op::Reply
        } else {
            return Err(Box::new(util::RuntimeError::new(format!("invalid operation: {}", op))))
        };
        let src_mac_addr = bytes.pop_mac_addr("src mac address")?;
        let src_ip_addr = bytes.pop_ipv4_addr("src ip address")?;
        let dst_mac_addr = bytes.pop_mac_addr("dst mac address")?;
        let dst_ip_addr = bytes.pop_ipv4_addr("dst ip address")?;

        Ok(Box::new(Frame {
            op: op,
            src_mac_addr: src_mac_addr,
            src_ip_addr: src_ip_addr,
            dst_mac_addr: dst_mac_addr,
            dst_ip_addr: dst_ip_addr,
        }))
    }
    fn to_bytes(self) -> frame::Bytes {
        use std::convert::TryInto;
        let mut bytes  = frame::Bytes::new(FRAME_SIZE);
        bytes.push_u16(HARDWARE_TYPE_ETHERNET);
        bytes.push_u16(ethernet::Type::Ipv4 as u16);
        bytes.push_u8(frame::MAC_ADDR_LEN.try_into().unwrap());
        bytes.push_u8(frame::IPV4_ADDR_LEN.try_into().unwrap());
        bytes.push_u16(self.op as u16);
        bytes.push_mac_addr(self.src_mac_addr);
        bytes.push_ipv4_addr(self.src_ip_addr);
        bytes.push_mac_addr(self.dst_mac_addr);
        bytes.push_ipv4_addr(self.dst_ip_addr);
        bytes
    }
}

struct Entry {
    pub ip_addr: frame::Ipv4Addr,
    pub mac_addr: frame::MacAddr,
    pub timestamp: DateTime<Utc>,
    pub cond: Condvar,
    pub data: Vec<u8>,
    pub interface: Arc<Mutex<ip::Interface>>,
}

lazy_static! {
    static ref TABLE: Arc<Mutex<Vec<Entry>>> = Arc::new(Mutex::new(vec![]));
    static ref TIMESTAMP: Mutex<DateTime<Utc>> = Mutex::new(Utc::now());
}

pub fn update_table(device: Arc<Mutex<dyn device::Device>>, ip_addr: &frame::Ipv4Addr, mac_addr: frame::MacAddr) -> Result<(), Box<dyn Error>> {
    let mut table = TABLE.lock().unwrap();
    let ref mut entry = table.iter_mut().find(|entry| &entry.ip_addr == ip_addr).ok_or(Box::new(util::RuntimeError::new(format!("can not entry with {}", ip_addr))))?;
    entry.mac_addr = mac_addr;
    entry.timestamp = Utc::now();
    if !entry.data.is_empty() {
        device.lock().unwrap().tx(ethernet::Type::Ipv4, entry.data.clone(), &entry.mac_addr.clone()); // TODO: do not clone entry.data
        entry.data = vec![];
    }
    entry.cond.notify_all();
    Ok(())
}

fn select(_ip_addr: &frame::Ipv4Addr) -> Option<Arc<Mutex<Entry>>> {
    unimplemented!()
}

fn send_request(_ip_interface: &Arc<Mutex<ip::Interface>>, _ip_addr: &frame::Ipv4Addr) -> Result<(), Box<dyn Error>> {
    unimplemented!()
}

fn table_freespace() -> Result<Arc<Mutex<Entry>>, Box<dyn Error>> {
    unimplemented!()
}

pub fn resolve(ip_interface: &Arc<Mutex<ip::Interface>>, ip_addr: frame::Ipv4Addr, data: &Vec<u8>) -> Result<Option<frame::MacAddr>, Box<dyn Error>> {
    match select(&ip_addr) {
        Some(entry) => {
            let lock = entry.lock().unwrap();
            if lock.mac_addr == ethernet::ADDR_ANY {
                send_request(ip_interface, &ip_addr)?;
                lock.cond.wait_timeout_until(
                    unimplemented!(), // TODO
                    ::std::time::Duration::from_secs(1),
                    |_: &mut i32| errno() != Errno::EINTR as i32)?;

                if  errno() == Errno::ETIMEDOUT as i32 {
                    entry_clear(&lock);
                    return Err(Box::new(util::RuntimeError::new("timed out".to_string())))
                }
            }
            Ok(Some(lock.mac_addr.clone()))
        },
        None => {
            let entry = table_freespace()?;
            entry.lock().unwrap().data = data.clone();
            entry.lock().unwrap().ip_addr = ip_addr.clone();
            entry.lock().unwrap().timestamp = Utc::now();
            entry.lock().unwrap().interface = Arc::clone(ip_interface);
            send_request(ip_interface, &ip_addr)?;
            Ok(None)
        }
    }
}

fn entry_clear(_entry: &Entry) {
    unimplemented!()
}

fn table_patrol() {
    unimplemented!()
}

pub fn rx(packet: &Vec<u8>, device: Arc<Mutex<dyn device::Device>>) {
    use frame::Frame;
    let frame = self::Frame::from_bytes(frame::Bytes::from_vec(packet.clone())).unwrap();
    eprintln!(">>> arp rx <<<");
    frame.dump();
    let now = Utc::now();
    if (now - *TIMESTAMP.lock().unwrap()).num_seconds() > 10 {
        *TIMESTAMP.lock().unwrap() = now;
        table_patrol();
    }
    let _marge = update_table(device, &frame.src_ip_addr, frame.src_mac_addr);
    unimplemented!()
}

