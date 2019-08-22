use bitflags::bitflags;
use std::error::Error;

pub struct Net {
    devices: Vec<Device>,
    drivers: Vec<DeviceDriver>,
    protos: Vec<DeviceProto>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    Ethernet,
    Slip,
}

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

pub struct Device {
    interfaces: Vec<Interface>,
    name: String,
    type_: DeviceType,
    mtu: u16,
    flags: DeviceFlags,
    hlen: u16,
    alen: u16,
    addr: [u8; 16],
    peer: [u8; 16],
    broadcast: [u8; 16],
    rx_handler: fn(&mut Net, &Device, DeviceProtoType, &Vec<u8>, usize),
    ops: DeviceOps,
    r#priv: Box<u8>,
}

impl Device {
    fn from_driver(driver: &DeviceDriver) -> Self {
        Device {
            interfaces: vec![],
            name: String::new(),
            type_: driver.type_,
            mtu: driver.mtu,
            flags: driver.flags,
            hlen: driver.hlen,
            alen: driver.alen,
            addr: [0; 16],
            peer: [0; 16],
            broadcast: [0; 16],
            rx_handler: rx_handler,
            ops: driver.ops,
            r#priv: Box::new(0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceFamily {
    Ipv4,
    Ipv6,
}

pub struct Interface {
    pub family: u8,
    pub device: Box<Device>,
}

#[derive(Clone, Copy)]
pub struct DeviceOps {
    pub open: fn(&Device, i32) -> Result<(), Box<dyn Error>>,
    pub close: fn(&Device) -> Result<(), Box<dyn Error>>,
    pub run: fn(&Device) -> Result<(), Box<dyn Error>>,
    pub stop: fn(&Device) -> Result<(), Box<dyn Error>>,
    pub tx: fn(&Device, u16, &Vec<u8>, usize, &Vec<u8>) -> Result<(), Box<dyn Error>>,
}

pub struct DeviceDriver {
    pub type_: DeviceType,
    pub mtu: u16,
    pub flags: DeviceFlags,
    pub hlen: u16,
    pub alen: u16,
    pub ops: DeviceOps,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceProtoType {
    Ip,
    Arp,
    Ipv6,
}

pub struct DeviceProto {
    pub type_: DeviceProtoType,
    pub handler: fn(packet: &Vec<u8>, plen: usize, dev: &Device),
}

impl Net {
    pub fn new() -> Self {
        Net {
            drivers: vec![],
            protos: vec![],
            devices: vec![],
        }
    }

    pub fn regist_driver(&mut self, driver: DeviceDriver) -> Result<(), Box<dyn Error>> {
        if self.drivers.iter().any(|d| driver.type_ == d.type_) {
            unimplemented!()
        }
        self.drivers.push(driver);
        Ok(())
    }

    pub fn regist_proto(&mut self, proto: DeviceProto) -> Result<(), Box<dyn Error>> {
        if self.protos.iter().any(|p| proto.type_ == p.type_) {
            unimplemented!()
        }
        self.protos.push(proto);
        Ok(())
    }

    pub fn alloc(&mut self, type_: DeviceType) -> Option<&Device> {
        let driver = self.drivers.iter().find(|driver| driver.type_ == type_)?;
        self.devices.push(Device::from_driver(driver));
        self.devices.last()
    }
}

fn rx_handler(
    net: &mut Net,
    device: &Device,
    type_: DeviceProtoType,
    packet: &Vec<u8>,
    plen: usize,
) {
    for proto in net.protos.iter() {
        if proto.type_ == type_ {
            // TODO : hton16
            (proto.handler)(packet, plen, device);
            return;
        }
    }
}
