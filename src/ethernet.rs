use std::error::Error;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::thread;

use bitflags::bitflags;

use crate::{frame, raw, util, interface};

pub const ADDR_LEN: usize = 6;
pub const ADDR_STR_LEN: usize = 18;

pub const HDR_SIZE: usize = 14;
pub const TRL_SIZE: usize = 4;
pub const FRAME_SIZE_MIN: usize = 64;
pub const FRAME_SIZE_MAX: usize = 1518;
pub const PAYLOAD_SIZE_MIN: usize = FRAME_SIZE_MIN - (HDR_SIZE + TRL_SIZE);
pub const PAYLOAD_SIZE_MAX: usize = FRAME_SIZE_MAX - (HDR_SIZE + TRL_SIZE);

pub const ADDR_ANY: frame::MacAddr = frame::MacAddr([0; ADDR_LEN]);
pub const ADDR_BROADCAST: frame::MacAddr = frame::MacAddr([255; ADDR_LEN]);

bitflags! {
    pub struct DeviceFlags: u32 {
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
    Ipv4 = 0x0800,
    Ipv6 = 0x86DD,
}

impl Type {
    pub fn from_u16(n: u16) -> Option<Type> {
        if n == Type::Arp as u16 {
            Some(Type::Arp)
        } else if n == Type::Ipv4 as u16 {
            Some(Type::Ipv4)
        } else if n == Type::Ipv6 as u16 {
            Some(Type::Ipv6)
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
                Type::Ipv4 => "IP",
                Type::Ipv6 => "IPv6",
            }
        )
    }
}

pub struct Frame {
    pub dst_addr: frame::MacAddr,
    pub src_addr: frame::MacAddr,
    pub type_: Type,
    pub data: frame::Bytes,
}

impl Frame {
    pub fn dump(&self) {
        eprintln!("dst : {}", self.dst_addr);
        eprintln!("src : {}", self.src_addr);
        eprintln!("type: {}", self.type_);
        eprintln!("{}", self.data);
    }
}

impl frame::Frame for Frame {
    fn from_bytes(mut bytes: frame::Bytes) -> Result<Box<Self>, Box<dyn Error>> {
        let dst_addr = bytes.pop_mac_addr("dst addr")?;
        let src_addr = bytes.pop_mac_addr("src addr")?;
        eprintln!("{}", bytes);
        let n = bytes.pop_u16("type")?;
        let type_ = Type::from_u16(n).ok_or(Box::new(util::RuntimeError::new(format!(
            "{} can not be EthernetType",
            n
        ))))?;
        Ok(Box::new(Frame {
            dst_addr: dst_addr,
            src_addr: src_addr,
            type_: type_,
            data: bytes,
        }))
    }
    fn to_bytes(self) -> frame::Bytes {
        let mut bytes = frame::Bytes::new(FRAME_SIZE_MAX);
        bytes.push_mac_addr(self.dst_addr);
        bytes.push_mac_addr(self.src_addr);
        bytes.push_u16(self.type_ as u16);
        bytes
    }
}

lazy_static! {
    static ref JOIN_HANDLES: Mutex<HashMap<String, thread::JoinHandle<()>>> = Mutex::new(HashMap::new());
}

#[derive(Debug, Clone)]
pub struct DeviceImpl {
    pub interfaces: Vec<Arc<Mutex<dyn interface::Interface + Send>>>,
    pub name: String,
    pub raw: Arc<Mutex<dyn raw::RawDevice + Send>>,
    pub addr: frame::MacAddr,
    pub broadcast_addr: frame::MacAddr,
    pub terminate: bool,
}

#[derive(Debug, Clone)]
pub struct Device(pub Arc<Mutex<DeviceImpl>>);

impl Device {
    pub fn open(name: &str, raw_type: raw::Type) -> Result<Device, Box<dyn Error>> {
        eprintln!("open : {}", name);
        Ok(Device(Arc::new(Mutex::new(DeviceImpl {
            interfaces: vec![],
            name: name.to_string(),
            raw: raw::open(raw_type, name),
            addr: ADDR_ANY.clone(),
            broadcast_addr: ADDR_BROADCAST.clone(),
            terminate: false,
        }))))
    }

    pub fn  close(self) -> Result<(), Box<dyn Error>> {
        let name = { self.0.lock().unwrap().name.clone() };
        eprintln!("close : {}", name);
        if let Some(handle) =  JOIN_HANDLES.lock().unwrap().remove(&name) {
            {
                self.0.lock().unwrap().terminate = true;
            }
            handle.join().unwrap();
        }
        self.0.lock().unwrap().raw.lock().unwrap().close()?;
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let device = self.clone();
        let name = device.0.lock().unwrap().name.clone();
        eprintln!("run : {}", name);
        let join_handle = thread::spawn(move || {
            loop {
                let device_ = device.clone();
                let (terminate, name, raw) = {
                    let device_inner = device.0.lock().unwrap();
                    (device_inner.terminate, device_inner.name.clone(), device_inner.raw.clone())
                };
                if terminate {
                    break;
                }
                raw.lock().unwrap().rx(
                    Box::new(move |buf: &Vec<u8>| device_.rx(buf).unwrap()),
                    1000,
                );
            }
        });
        JOIN_HANDLES.lock().unwrap().insert(name, join_handle);
        Ok(())
    }

    pub fn stop(&self) {
        unimplemented!()
    }

    pub fn tx(&mut self, _type_: Type, _payload: Vec<u8>, _dst: &frame::MacAddr) {
        unimplemented!()
    }

    pub fn rx(&self, frame: &Vec<u8>) -> Result<(), Box<dyn Error>> {
        use frame::Frame;
        let frame = self::Frame::from_bytes(frame::Bytes::from_vec(frame.clone()))?;
        frame.dump();

        // TODO: transfer data to upper protocol
        Ok(())
    }

    pub fn add_interface(&mut self, interface: Arc<Mutex<dyn interface::Interface + Send>>) -> Result<(), Box<dyn Error>> {
        let mut device = self.0.lock().unwrap();
        let family = interface.lock().unwrap().family();
        if device.interfaces.iter().any(|interface_| interface_.lock().unwrap().family() == family) {
            Err(Box::new(util::RuntimeError::new(format!("interface {} already exists at device {}", family, device.name))))
        } else {
            device.interfaces.push(interface);
            Ok(())
        }
    }
}

