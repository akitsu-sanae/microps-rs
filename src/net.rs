use bitflags::bitflags;
use std::any::Any;
use std::error::Error;

use crate::ethernet;
use crate::raw;

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
    pub interfaces: Vec<Interface>,
    pub name: String,
    pub type_: DeviceType,
    pub mtu: u16,
    pub flags: DeviceFlags,
    pub hlen: u16,
    pub alen: usize,
    pub addr: [u8; ethernet::ADDR_LEN],
    pub peer: [u8; ethernet::ADDR_LEN],
    pub broadcast: [u8; ethernet::ADDR_LEN],
    pub rx_handler: fn(&mut Net, &Device, DeviceProtoType, &Vec<u8>, usize),
    pub ops: DeviceOps,
    pub data: Box<dyn Any>,
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
            addr: [0; ethernet::ADDR_LEN],
            peer: [0; ethernet::ADDR_LEN],
            broadcast: [0; ethernet::ADDR_LEN],
            rx_handler: rx_handler,
            ops: driver.ops,
            data: Box::new(0),
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
    pub open: fn(&mut Device, raw::Type) -> Result<(), Box<dyn Error>>,
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
    pub alen: usize,
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

    pub fn alloc(&mut self, type_: DeviceType) {
        let driver = self
            .drivers
            .iter()
            .find(|driver| driver.type_ == type_)
            .unwrap();
        self.devices.push(Device::from_driver(driver));
    }

    pub fn last_device_mut(&mut self) -> Option<&mut Device> {
        self.devices.last_mut()
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
