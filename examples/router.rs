#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate microps_rs;
extern crate nix;

use microps_rs::{ethernet, raw, ipv4::{set_forwarding, Interface, InterfaceImpl}, frame};
use nix::sys::signal::{self, SigHandler, Signal};
use std::sync::atomic::{AtomicBool, Ordering};

lazy_static! {
    static ref TERMINATE: AtomicBool = AtomicBool::new(false);
}

extern "C" fn handle_sigint(signal: libc::c_int) {
    let signal = Signal::from_c_int(signal).unwrap();
    TERMINATE.store(signal == Signal::SIGINT, Ordering::Relaxed);
}

struct InterfaceData {
    pub name: &'static str,
    pub mac_addr: &'static str,
    pub ip_addr: &'static str,
    pub netmask: &'static str,
}

const INTERFACES: [InterfaceData; 2] = [
    InterfaceData {
        name: "tap0",
        mac_addr: "00:00:5E:00:53:00",
        ip_addr: "172.16.0.1",
        netmask: "255.255.255.0",
    },
    InterfaceData {
        name: "tap10",
        mac_addr: "00:00:5E:00:53:10",
        ip_addr: "172.16.1.1",
        netmask: "255.255.255.0",
    }
];


fn main() {
    set_forwarding(true);
    for interface in INTERFACES.iter() {
        let mut device = ethernet::Device::open(
            interface.name,
            frame::MacAddr::from_str(&interface.mac_addr.to_string()).unwrap(),
            raw::Type::Auto).unwrap();
        let interface = Interface::new(InterfaceImpl {
            device: device.clone(),
            unicast: frame::Ipv4Addr::from_str(interface.ip_addr.to_string()).unwrap(),
            netmask: frame::Ipv4Addr::from_str(interface.netmask.to_string()).unwrap(),
            gateway: frame::Ipv4Addr::empty(),
        });
        device.add_interface(interface).unwrap();
        device.run().unwrap();
        let name = {
            let device = device.0.lock().unwrap();
            device.name.clone()
        };
        eprintln!("[{}]", name);
    }

    let handler = SigHandler::Handler(handle_sigint);
    unsafe { signal::signal(Signal::SIGINT, handler) }.unwrap();

    while !TERMINATE.load(Ordering::SeqCst) {}
}
