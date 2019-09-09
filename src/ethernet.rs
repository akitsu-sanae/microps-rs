use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::{frame, raw, util, device, interface};

pub const ADDR_LEN: usize = 6;
pub const ADDR_STR_LEN: usize = 18;

pub const HDR_SIZE: usize = 14;
pub const TRL_SIZE: usize = 4;
pub const FRAME_SIZE_MIN: usize = 64;
pub const FRAME_SIZE_MAX: usize = 1518;
pub const PAYLOAD_SIZE_MIN: usize = FRAME_SIZE_MIN - (HDR_SIZE + TRL_SIZE);
pub const PAYLOAD_SIZE_MAX: usize = FRAME_SIZE_MAX - (HDR_SIZE + TRL_SIZE);

const ADDR_ANY: frame::MacAddr = frame::MacAddr([0; ADDR_LEN]);
const ADDR_BROADCAST: frame::MacAddr = frame::MacAddr([255; ADDR_LEN]);

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
}

impl frame::Frame for Frame {
    fn from_bytes(mut bytes: frame::Bytes) -> Result<Box<Self>, Box<dyn Error>> {
        let dst_addr = bytes.pop_mac_addr("dst addr")?;
        let src_addr = bytes.pop_mac_addr("src addr")?;
        let n = bytes.pop_u16("type")?;
        let type_ = Type::from_u16(n).ok_or(Box::new(util::RuntimeError::new(format!(
            "{} can not be EthernetType",
            n
        ))))?;
        Ok(Box::new(Frame {
            dst_addr: dst_addr,
            src_addr: src_addr,
            type_: type_,
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

pub struct EthernetDevice {
    pub interfaces: Vec<Arc<Mutex<dyn interface::Interface + Send>>>,
    pub name: String,
    pub raw: Arc<Mutex<dyn raw::RawDevice + Send>>,
    pub addr: frame::MacAddr,
    pub broadcast_addr: frame::MacAddr,
    pub join_handle: Option<thread::JoinHandle<()>>,
    pub terminate: bool,
}

impl EthernetDevice {
    pub fn open(name: &str, raw_type: raw::Type) -> Result<EthernetDevice, Box<dyn Error>> {
        Ok(EthernetDevice {
            interfaces: vec![],
            name: name.to_string(),
            raw: raw::open(raw_type, name),
            addr: ADDR_ANY.clone(),
            broadcast_addr: ADDR_BROADCAST.clone(),
            join_handle: None,
            terminate: false,
        })
    }

    fn rx(&self, _frame: &Vec<u8>) {
        unimplemented!()
    }
}

impl device::Device for Arc<Mutex<EthernetDevice>> {
    fn name(&self) -> String {
        self.lock().unwrap().name.clone()
    }
    fn add_interface(&mut self, interface: Arc<Mutex<dyn interface::Interface + Send>>) -> Result<(), Box<dyn Error>> {
        let family = interface.lock().unwrap().family();
        let mut device = self.lock().unwrap();
        if  device.interfaces.iter().any(|interface_| interface_.lock().unwrap().family() == family) {
            Err(Box::new(util::RuntimeError::new(format!("interface {} already exists at device {}", family, device.name))))
        } else {
            device.interfaces.push(interface);
            Ok(())
        }
    }
    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let device_ = Arc::clone(self);
        self.lock().unwrap().join_handle = Some(thread::spawn(move || {
            while device_.lock().unwrap().terminate {
                let device__ = Arc::clone(&device_);
                device_.lock().unwrap().raw.lock().unwrap().rx(
                    Box::new(move |buf: &Vec<u8>| device__.lock().unwrap().rx(buf)),
                    1000,
                    );
            }
        }));
        Ok(())
    }

    fn  close(self) -> Result<(), Box<dyn Error>> {
        if let Ok(device) = Arc::try_unwrap(self) {
            let mut device = device.lock().unwrap();
            if let Some(handle) = device.join_handle.take() {
                device.terminate = true;
                handle.join().unwrap();
                device.raw.lock().unwrap().close()?;
            } else {
                device.raw.lock().unwrap().close()?;
            }
            Ok(())
        } else {
            Err(Box::new(util::RuntimeError::new("cannot close because of having multiple references".to_string())))
        }
    }
}

