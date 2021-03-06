#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate microps_rs;
extern crate nix;

use microps_rs::{ethernet, ip, raw};
use nix::sys::signal::{self, SigHandler, Signal};
use std::sync::atomic::{AtomicBool, Ordering};

lazy_static! {
    static ref TERMINATE: AtomicBool = AtomicBool::new(false);
}

extern "C" fn handle_sigint(signal: libc::c_int) {
    let signal = Signal::from_c_int(signal).unwrap();
    TERMINATE.store(signal == Signal::SIGINT, Ordering::Relaxed);
}

fn main() {
    let args: Vec<String> = ::std::env::args().into_iter().collect();
    let (ifname, mac_addr, ip_addr, netmask) = if args.len() == 4 {
        (
            args[1].clone(),
            None,
            ip::Addr::from_str(&args[2]).unwrap(),
            ip::Addr::from_str(&args[3]).unwrap(),
        )
    } else if args.len() == 5 {
        (
            args[1].clone(),
            Some(ethernet::MacAddr::from_str(&args[2]).unwrap()),
            ip::Addr::from_str(&args[3]).unwrap(),
            ip::Addr::from_str(&args[4]).unwrap(),
        )
    } else {
        panic!("USAGE: icmp_test <interface> [mac_address] <ip_address> <netmask>");
    };

    let handler = SigHandler::Handler(handle_sigint);
    unsafe { signal::signal(Signal::SIGINT, handler) }.unwrap();

    let mut device = ethernet::Device::open(
        ifname.as_str(),
        match mac_addr {
            None => ethernet::ADDR_ANY,
            Some(mac_addr) => mac_addr,
        },
        raw::Type::Auto,
    )
    .unwrap();
    eprintln!("ip_addr: {}", ip_addr);
    let interface = ip::interface::Interface::new(device.clone(), ip_addr, netmask, None);
    device.add_interface(interface);
    device.run().unwrap();
    eprintln!("[{}]", ifname);
    while !TERMINATE.load(Ordering::SeqCst) {}
    device.close().unwrap();
}
