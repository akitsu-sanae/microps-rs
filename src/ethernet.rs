use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;

use arrayvec::ArrayVec;
use bitflags::bitflags;

use crate::{arp, buffer::Buffer, ip, packet, raw, util::RuntimeError};

mod frame;

pub const ADDR_LEN: usize = 6;
pub const ADDR_STR_LEN: usize = 18;

pub const HDR_SIZE: usize = 14;
pub const TRL_SIZE: usize = 4;
pub const FRAME_SIZE_MIN: usize = 64;
pub const FRAME_SIZE_MAX: usize = 1518;
pub const PAYLOAD_SIZE_MIN: usize = FRAME_SIZE_MIN - (HDR_SIZE + TRL_SIZE);
pub const PAYLOAD_SIZE_MAX: usize = FRAME_SIZE_MAX - (HDR_SIZE + TRL_SIZE);

pub const ADDR_ANY: MacAddr = MacAddr([0; ADDR_LEN]);
pub const ADDR_BROADCAST: MacAddr = MacAddr([255; ADDR_LEN]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacAddr(pub [u8; ADDR_LEN]);

impl MacAddr {
    pub fn empty() -> MacAddr {
        MacAddr([0; ADDR_LEN])
    }
    pub fn from_str(str: &String) -> Result<Self, Box<dyn Error>> {
        str.split(':')
            .map(|n| u8::from_str_radix(n, 16))
            .collect::<Result<ArrayVec<[_; 6]>, _>>()
            .map(|arr| Self(arr.into_inner().unwrap()))
            .or_else(|err| Err(RuntimeError::new(format!("{}", err))))
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

bitflags! {
    pub struct DeviceFlags: u32 {
        const EMPTY = 0x00;
        const BROADCAST = 0x01;
        const MULTICAST = 0x02;
        const P2P       = 0x04;
        const LOOPBACK  = 0x08;
        const NOARP     = 0x10;
        const PROMISC   = 0x20;
        const RUNNING   = 0x40;
        const UP        = 0x80;
    }
}

#[repr(u16)]
pub enum Type {
    Arp = 0x0806,
    Ip = 0x0800,
}

impl Type {
    pub fn from_u16(n: u16) -> Option<Type> {
        if n == Type::Arp as u16 {
            Some(Type::Arp)
        } else if n == Type::Ip as u16 {
            Some(Type::Ip)
        } else {
            None
        }
    }
}

use std::fmt;
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Type::Arp => "ARP",
                Type::Ip => "IP",
            }
        )
    }
}

lazy_static! {
    static ref JOIN_HANDLES: Mutex<HashMap<String, thread::JoinHandle<()>>> =
        Mutex::new(HashMap::new());
}

#[derive(Debug, Clone)]
pub struct DeviceImpl {
    pub interface: Option<ip::interface::Interface>,
    pub name: String,
    pub raw: Arc<dyn raw::RawDevice + Sync + Send>,
    pub addr: MacAddr,
    pub broadcast_addr: MacAddr,
    pub terminate: bool,
}

#[derive(Debug, Clone)]
pub struct Device(pub Arc<Mutex<DeviceImpl>>);

impl Device {
    pub fn open(
        name: &str,
        mut addr: MacAddr,
        raw_type: raw::Type,
    ) -> Result<Device, Box<dyn Error>> {
        let raw = raw::open(raw_type, name);
        if addr == ADDR_ANY {
            addr = { raw.addr()? };
        }
        Ok(Device(Arc::new(Mutex::new(DeviceImpl {
            interface: None,
            name: name.to_string(),
            raw: raw,
            addr: addr,
            broadcast_addr: ADDR_BROADCAST.clone(),
            terminate: false,
        }))))
    }

    pub fn close(self) -> Result<(), Box<dyn Error>> {
        let name = { self.0.lock().unwrap().name.clone() };
        if let Some(handle) = JOIN_HANDLES.lock().unwrap().remove(&name) {
            {
                self.0.lock().unwrap().terminate = true;
            }
            handle.join().unwrap();
        }
        self.0.lock().unwrap().raw.close()?;
        Ok(())
    }

    pub fn add_interface(&mut self, interface: ip::interface::Interface) {
        let mut inner = self.0.lock().unwrap();
        inner.interface = Some(interface);
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let device = self.clone();
        let name = device.0.lock().unwrap().name.clone();
        let join_handle = thread::spawn(move || loop {
            let device_ = device.clone();
            let (terminate, raw) = {
                let device_inner = device.0.lock().unwrap();
                (device_inner.terminate, device_inner.raw.clone())
            };
            if terminate {
                break;
            }
            let tx_handle = raw.rx(Box::new(move |buf: Buffer| device_.rx(buf)), 1000);
            match tx_handle {
                Ok(Some(tx_join_handle)) => {
                    tx_join_handle.join().unwrap();
                }
                Ok(None) => {}
                Err(_err) => {} // TODO: use err
            }
        });
        JOIN_HANDLES.lock().unwrap().insert(name, join_handle);
        Ok(())
    }

    pub fn tx(
        &self,
        type_: Type,
        payload: Buffer,
        dst_addr: MacAddr,
    ) -> Result<(), Box<dyn Error>> {
        let device_inner = self.0.lock().unwrap();
        let src_addr = device_inner.addr.clone();
        let frame = frame::Frame {
            dst_addr: dst_addr,
            src_addr: src_addr,
            type_: type_,
            payload: payload,
        };
        use packet::Packet;
        device_inner.raw.tx(frame.to_buffer())
    }

    pub fn rx(&self, buffer: Buffer) -> Result<Option<thread::JoinHandle<()>>, Box<dyn Error>> {
        use packet::Packet;
        let frame = frame::Frame::from_buffer(buffer)?;

        if cfg!(debug_assertions) {
            eprintln!(">>> ethernet_rx <<<");
            frame.dump();
        }
        let type_ = frame.type_;
        let payload = frame.payload;
        self.rx_handler(type_, payload)
    }

    fn rx_handler(
        &self,
        type_: Type,
        payload: Buffer,
    ) -> Result<Option<thread::JoinHandle<()>>, Box<dyn Error>> {
        match type_ {
            Type::Arp => arp::rx(payload, self),
            Type::Ip => ip::rx(payload, self),
        }
    }
}
