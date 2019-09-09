extern crate arrayvec;
extern crate bitflags;
extern crate hexdump;
extern crate ifstructs;
extern crate libc;

#[macro_use]
extern crate nix;

use std::error::Error;

pub mod arp;
pub mod ethernet;
pub mod frame;
pub mod icmp;
pub mod ip;
pub mod device;
pub mod interface;
pub mod net;
pub mod raw;
pub mod slip;
pub mod tcp;
pub mod udp;
pub mod util;

use std::sync::{Arc, Mutex};

pub struct Microps {
    pub devices: Vec<Arc<Mutex<dyn device::Device>>>,
}

impl Microps {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Microps {
            devices: vec![],
        })
    }
}

impl Drop for Microps {
    fn drop(&mut self) {}
}
