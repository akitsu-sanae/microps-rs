use arrayvec::ArrayVec;
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;

use crate::{
    buffer::Buffer,
    ethernet, icmp,
    ip::interface::Interface,
    packet,
    protocol::{ProtocolType, PROTOCOLS},
    util,
};

pub mod dgram;
mod fragment;
pub mod interface;
mod route;

pub const VERSION: u8 = 4;

pub const ADDR_LEN: usize = 4;
const ADDR_ANY: Addr = Addr([0; ADDR_LEN]);
const ADDR_BROADCAST: Addr = Addr([255; ADDR_LEN]);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Addr(pub [u8; ADDR_LEN]);

impl Addr {
    pub fn empty() -> Self {
        Addr([0; ADDR_LEN])
    }
    pub fn full() -> Self {
        Addr([0xff; ADDR_LEN])
    }

    pub fn from_str(str: &String) -> Result<Self, Box<dyn Error>> {
        str.split('.')
            .map(|n| u8::from_str_radix(n, 10))
            .collect::<Result<ArrayVec<[_; 4]>, _>>()
            .map(|arr| Self(arr.into_inner().unwrap()))
            .or_else(|err| Err(util::RuntimeError::new(format!("{}", err))))
    }

    pub fn apply_mask(&self, mask: &Addr) -> Addr {
        Addr([
            self.0[0] & mask.0[0],
            self.0[1] & mask.0[1],
            self.0[2] & mask.0[2],
            self.0[3] & mask.0[3],
        ])
    }
}

use std::ops::BitOr;
impl BitOr for Addr {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Addr([
            self.0[0] | rhs.0[0],
            self.0[1] | rhs.0[1],
            self.0[2] | rhs.0[2],
            self.0[3] | rhs.0[3],
        ])
    }
}

use std::ops::Not;
impl Not for Addr {
    type Output = Self;
    fn not(self) -> Self::Output {
        Addr([!self.0[0], !self.0[1], !self.0[2], !self.0[3]])
    }
}

use std::fmt;
impl fmt::Display for Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}.{}", self.0[0], self.0[1], self.0[2], self.0[3])
    }
}

lazy_static! {
    static ref IS_FORWARDING: AtomicBool = AtomicBool::new(false);
}

pub fn set_is_forwarding(b: bool) {
    IS_FORWARDING.store(b, Ordering::Relaxed);
}

fn forward_process(mut dgram: dgram::Dgram, interface: &Interface) -> Result<(), Box<dyn Error>> {
    use packet::Packet;
    if dgram.time_to_live != 1 {
        let src = dgram.src;
        icmp::tx(
            interface,
            icmp::Type::TimeExceeded,
            icmp::Code::Exceeded(icmp::CodeExceeded::Ttl),
            0,
            dgram.to_buffer(),
            &src,
        )?;
        return Err(util::RuntimeError::new(format!("time exceeded")));
    }
    let route = match route::lookup(Some(interface), dgram.dst) {
        Some(route) => route,
        None => {
            let src = dgram.src;
            icmp::tx(
                interface,
                icmp::Type::DestUnreach,
                icmp::Code::Unreach(icmp::CodeUnreach::Net),
                0,
                dgram.to_buffer(),
                &src,
            )?;
            return Err(util::RuntimeError::new(format!("destination unreach")));
        }
    };
    {
        let route_interface = route.interface.0.lock().unwrap();
        let route_device = route_interface.device.clone();
        if route_interface.unicast == dgram.dst {
            rx(dgram.to_buffer(), &route_device)?;
            return Ok(());
        }
    }
    if dgram.offset & 0x4000 != 0 && dgram.payload.0.len() > ethernet::PAYLOAD_SIZE_MAX {
        let src = dgram.src;
        return icmp::tx(
            interface,
            icmp::Type::DestUnreach,
            icmp::Code::Unreach(icmp::CodeUnreach::FragmentNeeded),
            0,
            dgram.to_buffer(),
            &src,
        );
        // return Err(util::RuntimeError::new(format!("destination unreach")));
    }
    dgram.time_to_live -= 1;
    let sum = dgram.checksum;
    dgram.checksum = util::calc_checksum(
        dgram
            .payload
            .head(((dgram.version_header_length & 0x0f) as usize) << 2),
        (u16::max_value() - dgram.checksum) as u32,
    );
    let ret = route.interface.tx_device(
        dgram.clone().to_buffer(), // TODO: remove clone if possible
        &match route.nexthop {
            Some(next) => Some(next),
            None => Some(dgram.src),
        },
    );
    match ret {
        Ok(()) => Ok(()),
        Err(_) => {
            // restore original IP header
            dgram.time_to_live += 1;
            dgram.checksum = sum;

            let src = dgram.src;

            icmp::tx(
                interface,
                icmp::Type::DestUnreach,
                match route.nexthop {
                    Some(_) => icmp::Code::Unreach(icmp::CodeUnreach::Net),
                    None => icmp::Code::Unreach(icmp::CodeUnreach::Host),
                },
                0,
                dgram.to_buffer(),
                &src,
            )
        }
    }
}

pub fn rx(
    dgram: Buffer,
    device: &ethernet::Device,
) -> Result<Option<JoinHandle<()>>, Box<dyn Error>> {
    use packet::Packet;
    let dgram = dgram::Dgram::from_buffer(dgram)?;
    let device = device.0.lock().unwrap();
    let interface = device
        .interface
        .as_ref()
        .ok_or(util::RuntimeError::new(format!(
            "device `{}` has not ip interface.",
            device.name
        )))?;
    let (unicast, broadcast) = {
        let interface = interface.0.lock().unwrap();
        let network = interface.unicast.apply_mask(&interface.netmask);
        let broadcast = network | !interface.netmask;
        (interface.unicast.clone(), broadcast)
    };
    if dgram.dst != unicast && dgram.dst != broadcast && dgram.dst != ADDR_BROADCAST {
        /* forward to other host */
        if IS_FORWARDING.load(Ordering::SeqCst) {
            forward_process(dgram, interface)?;
        }
        return Ok(None);
    }
    if cfg!(debug_assertions) {
        eprintln!(">>> ip tx <<<");
        dgram.dump();
    }

    let (src, dst, protocol_type) = (dgram.src, dgram.dst, dgram.protocol);
    let payload = if dgram.offset & 0x2000 != 0 || dgram.offset & 0x1ff != 0 {
        let fragment = fragment::process(dgram)?;
        fragment.data
    } else {
        dgram.payload
    };
    let protocols = PROTOCOLS.lock().unwrap();
    for protocol in protocols.iter() {
        if protocol.type_() == protocol_type {
            protocol.handler(payload, src, dst, interface)?;
            return Ok(None);
        }
    }
    Err(util::RuntimeError::new(format!("no suitable protocol")))
}
