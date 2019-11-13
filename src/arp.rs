mod table;

use std::error::Error;
use std::thread::{self, JoinHandle};

use chrono::Utc;
use nix::errno::{errno, Errno};

use crate::{
    ethernet,
    frame::{self, Frame},
    ipv4,
    util::RuntimeError,
};

const HARDWARE_TYPE_ETHERNET: u16 = 0x0001;

#[repr(u16)]
#[derive(Debug, PartialEq, Eq)]
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Op::Request => "Request",
                Op::Reply => "Reply",
            }
        )
    }
}

#[derive(Debug)]
pub struct ArpFrame {
    pub op: Op,
    pub src_mac_addr: frame::MacAddr,
    pub src_ip_addr: frame::Ipv4Addr,
    pub dst_mac_addr: frame::MacAddr,
    pub dst_ip_addr: frame::Ipv4Addr,
}

const FRAME_SIZE: usize = 52;

impl ArpFrame {
    pub fn dump(&self) {
        eprintln!("op: {}", self.op);
        eprintln!("src mac addr: {}", self.src_mac_addr);
        eprintln!("src ip addr: {}", self.src_ip_addr);
        eprintln!("dst mac addr: {}", self.dst_mac_addr);
        eprintln!("dst ip addr: {}", self.dst_ip_addr);
    }
}

impl frame::Frame for ArpFrame {
    fn from_bytes(mut bytes: frame::Bytes) -> Result<Box<Self>, Box<dyn Error>> {
        let hardware_type = bytes.pop_u16("hardware type")?;
        if hardware_type != HARDWARE_TYPE_ETHERNET {
            return Err(RuntimeError::new(format!(
                "hardware type must be {}, but {}",
                HARDWARE_TYPE_ETHERNET, hardware_type
            )));
        }

        let protocol = bytes.pop_u16("protocol type")?;
        if protocol != ethernet::Type::Ipv4 as u16 {
            return Err(RuntimeError::new(format!(
                "protocol type must be {}, but {}",
                ethernet::Type::Ipv4 as u16,
                protocol
            )));
        }

        let hardware_address_len = bytes.pop_u8("hardware address length")?;
        if hardware_address_len as usize != frame::MAC_ADDR_LEN {
            return Err(RuntimeError::new(format!(
                "hardware address length must be {}, but {}",
                frame::MAC_ADDR_LEN,
                hardware_address_len
            )));
        }

        let ip_address_len = bytes.pop_u8("ip address length")?;
        if ip_address_len as usize != frame::IPV4_ADDR_LEN {
            return Err(RuntimeError::new(format!(
                "ip address length must be {}, but {}",
                frame::IPV4_ADDR_LEN,
                ip_address_len
            )));
        }

        let op: u16 = bytes.pop_u16("operation")?;
        let op = if op == Op::Request as u16 {
            Op::Request
        } else if op == Op::Reply as u16 {
            Op::Reply
        } else {
            return Err(RuntimeError::new(format!("invalid operation: {}", op)));
        };
        let src_mac_addr = bytes.pop_mac_addr("src mac address")?;
        let src_ip_addr = bytes.pop_ipv4_addr("src ip address")?;
        let dst_mac_addr = bytes.pop_mac_addr("dst mac address")?;
        let dst_ip_addr = bytes.pop_ipv4_addr("dst ip address")?;

        Ok(Box::new(ArpFrame {
            op: op,
            src_mac_addr: src_mac_addr,
            src_ip_addr: src_ip_addr,
            dst_mac_addr: dst_mac_addr,
            dst_ip_addr: dst_ip_addr,
        }))
    }
    fn to_bytes(self) -> frame::Bytes {
        use std::convert::TryInto;
        let mut bytes = frame::Bytes::new(FRAME_SIZE);
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

fn update_table(
    ip_addr: &frame::Ipv4Addr,
    mac_addr: &frame::MacAddr,
) -> Result<(), Box<dyn Error>> {
    let idx = table::lookup(ip_addr).ok_or(RuntimeError::new(format!(
        "not found in table: {}",
        ip_addr
    )))?;
    let (data, device, mac_addr) = {
        let mut table = table::TABLE.lock().unwrap();
        let ref mut entry = table.get_mut(idx).unwrap();
        entry.mac_addr = mac_addr.clone();
        entry.timestamp = Utc::now();
        entry.cond.notify_all();
        let device = entry.interface.0.lock().unwrap().device.clone();
        (entry.data.clone(), device, entry.mac_addr.clone())
    };
    if data.is_empty() {
        device.tx(ethernet::Type::Ipv4, data, mac_addr)?;
        table::remove(idx)?;
    }
    Ok(())
}

fn send_request(
    interface: &ipv4::Interface,
    ip_addr: &frame::Ipv4Addr,
) -> Result<(), Box<dyn Error>> {
    let (src_mac_addr, src_ip_addr) = {
        let interface_inner = interface.0.lock().unwrap();
        let device_inner = interface_inner.device.0.lock().unwrap();
        (device_inner.addr.clone(), interface_inner.unicast.clone())
    };
    let request = ArpFrame {
        op: Op::Request,
        src_mac_addr: src_mac_addr,
        src_ip_addr: src_ip_addr,
        dst_mac_addr: frame::MacAddr::empty(),
        dst_ip_addr: ip_addr.clone(),
    };
    let device = {
        let interface_inner = interface.0.lock().unwrap();
        interface_inner.device.clone()
    };
    device.tx(
        ethernet::Type::Arp,
        request.to_bytes(),
        ethernet::ADDR_BROADCAST.clone(),
    )?;
    Ok(())
}

fn send_reply(
    interface: ipv4::Interface,
    mac_addr: frame::MacAddr,
    ip_addr: frame::Ipv4Addr,
    dst_addr: frame::MacAddr,
) -> Result<Option<JoinHandle<()>>, Box<dyn Error>> {
    Ok(Some(thread::spawn(move || {
        let (src_mac_addr, src_ip_addr, device) = {
            let interface_inner = interface.0.lock().unwrap();
            let addr = interface_inner.device.0.lock().unwrap().addr.clone();
            (
                addr,
                interface_inner.unicast.clone(),
                interface_inner.device.clone(),
            )
        };
        let reply = ArpFrame {
            op: Op::Reply,
            src_mac_addr: src_mac_addr,
            src_ip_addr: src_ip_addr,
            dst_mac_addr: mac_addr,
            dst_ip_addr: ip_addr,
        };
        eprintln!(">>> arp reply <<<");
        reply.dump();
        device
            .tx(ethernet::Type::Arp, reply.to_bytes(), dst_addr)
            .unwrap();
        ()
    })))
}

pub fn resolve(
    ip_interface: &ipv4::Interface,
    ip_addr: frame::Ipv4Addr,
    data: frame::Bytes,
) -> Result<Option<frame::MacAddr>, Box<dyn Error>> {
    match table::lookup(&ip_addr) {
        Some(idx) => {
            let (timeout, mac_addr) = {
                let table = table::TABLE.lock().unwrap();
                let ref entry = table.get(idx).unwrap();
                if entry.mac_addr == ethernet::ADDR_ANY {
                    send_request(ip_interface, &ip_addr)?;
                    while {
                        entry.cond.wait_timeout(
                            table::TABLE.lock().unwrap(),
                            ::std::time::Duration::from_secs(1),
                        )?;
                        errno() == Errno::EINTR as i32
                    } {}
                    if errno() == Errno::ETIMEDOUT as i32 {
                        (Some(idx), entry.mac_addr.clone())
                    } else {
                        (None, entry.mac_addr.clone())
                    }
                } else {
                    (None, entry.mac_addr.clone())
                }
            };
            if let Some(idx) = timeout {
                table::remove(idx)?;
                return Err(RuntimeError::new("timed out".to_string()));
            }
            Ok(Some(mac_addr))
        }
        None => {
            let mut new_entry = table::Entry::new(
                ip_addr.clone(),
                frame::MacAddr::empty(),
                ip_interface.clone(),
            );
            new_entry.data = data;
            table::TABLE.lock().unwrap().push(new_entry);
            send_request(ip_interface, &ip_addr)?;
            Ok(None)
        }
    }
}

pub fn rx(
    packet: frame::Bytes,
    device: &ethernet::Device,
) -> Result<Option<JoinHandle<()>>, Box<dyn Error>> {
    let message = self::ArpFrame::from_bytes(packet).unwrap();
    table::patrol();
    let marge = update_table(&message.src_ip_addr, &message.src_mac_addr).is_ok();
    let interface = device.get_interface()?; // TODO: specify the kind of interface
    let src_ip_addr = interface.0.lock().unwrap().unicast.clone();
    if src_ip_addr == message.dst_ip_addr {
        if !marge {
            let mut table = table::TABLE.lock().unwrap();
            table.push(table::Entry::new(
                message.src_ip_addr.clone(),
                message.src_mac_addr.clone(),
                interface.clone(),
            ));
        }
        if message.op == Op::Request {
            let interface = interface.clone();
            let src_mac_addr = message.src_mac_addr;
            let src_ip_addr = message.src_ip_addr;
            let src_mac_addr_ = src_mac_addr.clone();
            send_reply(interface, src_mac_addr, src_ip_addr, src_mac_addr_)?;
        }
    }
    Ok(None)
}
