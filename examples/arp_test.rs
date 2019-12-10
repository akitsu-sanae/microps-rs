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
    let (ifname, mac_addr, ip_addr) = if args.len() == 3 {
        (args[1].clone(), None, args[2].clone())
    } else if args.len() == 4 {
        (args[1].clone(), Some(args[2].clone()), args[3].clone())
    } else {
        panic!("USAGE: arp_test <interface> [mac_address] <ip_address>");
    };
    let handler = SigHandler::Handler(handle_sigint);
    unsafe { signal::signal(Signal::SIGINT, handler) }.unwrap();

    let mut device = ethernet::Device::open(
        ifname.as_str(),
        match mac_addr {
            None => ethernet::ADDR_ANY,
            Some(mac_addr) => ethernet::MacAddr::from_str(&mac_addr).unwrap(),
        },
        raw::Type::Auto,
    )
    .unwrap();
    eprintln!("ip_addr: {}", ip_addr);
    let interface = ip::interface::Interface::new(
        device.clone(),
        ip::Addr::from_str(&ip_addr).unwrap(),
        ip::Addr::empty(),
        ip::Addr::empty(),
    );
    device.add_interface(interface);
    device.run().unwrap();
    eprintln!("[{}]", ifname);
    while !TERMINATE.load(Ordering::SeqCst) {}
    device.close().unwrap();
}
