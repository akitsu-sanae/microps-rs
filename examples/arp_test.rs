#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate microps_rs;
extern crate nix;

use microps_rs::{ethernet, raw, ip, device::Device};
use nix::sys::signal::{self, SigHandler, Signal};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};

lazy_static! {
    static ref TERMINATE: AtomicBool = AtomicBool::new(false);
}

extern "C" fn handle_sigint(signal: libc::c_int) {
    let signal = Signal::from_c_int(signal).unwrap();
    TERMINATE.store(signal == Signal::SIGINT, Ordering::Relaxed);
}

fn main() {
    let args: Vec<String> = ::std::env::args().into_iter().collect();
    let (ifname, mac_addr, ip_addr) = if args.len() == 3 {
        (args[1].clone(), None, args[2].clone())
    } else if args.len() == 4 {
        (args[1].clone(), Some(args[2].clone()), args[3].clone())
    } else {
        panic!("USAGE: arp_test <interface> [mac_address] <ip_address>");
    };
    let handler = SigHandler::Handler(handle_sigint);
    unsafe { signal::signal(Signal::SIGINT, handler) }.unwrap();

    arp::init();

    let mut device = Arc::new(Mutex::new(ethernet::EthernetDevice::open(ifname.as_str(), raw::Type::Auto).unwrap()));
    let interface = Arc::new(Mutex::new(ip::Interface {
        // unicast: ip_addr,
    }));
    device.add_interface(interface);
    device.run();
    eprintln!("[{}]", device.name());
    while !TERMINATE.load(Ordering::SeqCst) {}
    device.close().unwrap();
}
