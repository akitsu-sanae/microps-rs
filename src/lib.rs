extern crate bitflags;

use std::error::Error;

pub mod arp;
pub mod ethernet;
pub mod icmp;
pub mod ip;
pub mod net;
pub mod slip;
pub mod tcp;
pub mod udp;
pub mod util;

pub struct Microps {
    pub net: net::Net,
    /*
    slip: slip::Slip,
    arp: arp::Arp,
    ip: ip::Ip,
    icmp: icmp::Icmp,
    udp: udp::Udp,
    tcp: tcp::Tcp, */
}

impl Microps {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let mut net = net::Net::new();
        ethernet::init(&mut net)?;
        Ok(Self {
            net: net,
            /*
            slip: slip::Slip::new()?,
            arp: arp::Slip::new()?,
            ip: ip::Ip::new()?,
            icmp: icmp::Icmp::new()?,
            udp: udp::Udp::new()?,
            tcp: tcp::Tcp::new()?, */
        })
    }
}

impl Drop for Microps {
    fn drop(&mut self) {}
}
