mod frame;
mod table;

use std::error::Error;
use std::thread::{self, JoinHandle};

use chrono::Utc;
use nix::errno::{errno, Errno};

use crate::{buffer::Buffer, ethernet, ip, packet::Packet, util::RuntimeError};

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

fn update_table(ip_addr: &ip::Addr, mac_addr: &ethernet::MacAddr) -> Result<(), Box<dyn Error>> {
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
        device.tx(ethernet::Type::Ip, data, mac_addr)?;
        table::remove(idx)?;
    }
    Ok(())
}

fn send_request(
    interface: &ip::interface::Interface,
    ip_addr: &ip::Addr,
) -> Result<(), Box<dyn Error>> {
    let (src_mac_addr, src_ip_addr) = {
        let interface_inner = interface.0.lock().unwrap();
        let device_inner = interface_inner.device.0.lock().unwrap();
        (device_inner.addr.clone(), interface_inner.unicast.clone())
    };
    let request = frame::Frame {
        op: Op::Request,
        src_mac_addr: src_mac_addr,
        src_ip_addr: src_ip_addr,
        dst_mac_addr: ethernet::MacAddr::empty(),
        dst_ip_addr: ip_addr.clone(),
    };
    let device = {
        let interface_inner = interface.0.lock().unwrap();
        interface_inner.device.clone()
    };
    eprintln!(">>> arp request <<<");
    request.dump();

    device.tx(
        ethernet::Type::Arp,
        request.to_buffer(),
        ethernet::ADDR_BROADCAST.clone(),
    )?;
    Ok(())
}

fn send_reply(
    interface: ip::interface::Interface,
    mac_addr: ethernet::MacAddr,
    ip_addr: ip::Addr,
    dst_addr: ethernet::MacAddr,
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
        let reply = frame::Frame {
            op: Op::Reply,
            src_mac_addr: src_mac_addr,
            src_ip_addr: src_ip_addr,
            dst_mac_addr: mac_addr,
            dst_ip_addr: ip_addr,
        };
        eprintln!(">>> arp reply <<<");
        reply.dump();
        device
            .tx(ethernet::Type::Arp, reply.to_buffer(), dst_addr)
            .unwrap();
        ()
    })))
}

pub fn resolve(
    ip_interface: &ip::interface::Interface,
    ip_addr: ip::Addr,
    data: Buffer,
) -> Result<Option<ethernet::MacAddr>, Box<dyn Error>> {
    match table::lookup(&ip_addr) {
        Some(idx) => {
            let (timeout, mac_addr) = {
                let table = table::TABLE.lock().unwrap();
                let ref entry = table.get(idx).unwrap();
                if entry.mac_addr == ethernet::ADDR_ANY {
                    send_request(ip_interface, &ip_addr)?;
                    while {
                        let _table = entry.cond.wait_timeout(
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
                ethernet::MacAddr::empty(),
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
    packet: Buffer,
    device: &ethernet::Device,
) -> Result<Option<JoinHandle<()>>, Box<dyn Error>> {
    let message = frame::Frame::from_buffer(packet).unwrap();
    eprintln!(">>> arp rx <<<");
    message.dump();
    table::patrol();
    let marge = update_table(&message.src_ip_addr, &message.src_mac_addr).is_ok();
    let device = device.0.lock().unwrap();
    let interface = device.interface.as_ref().ok_or(RuntimeError::new(format!(
        "device `{}` has not ip interface.",
        device.name
    )))?;
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
