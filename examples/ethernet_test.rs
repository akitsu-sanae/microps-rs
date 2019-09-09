extern crate microps_rs;
extern crate nix;

use microps_rs::{device::Device, ethernet, raw};
use nix::sys::signal;
use std::sync::{Arc, Mutex};

fn main() {
    let args: Vec<String> = ::std::env::args().into_iter().collect();
    if args.len() != 2 {
        panic!("USAGE: ethernet_test <device>");
    }
    let mut sigset = signal::SigSet::empty();
    sigset.add(signal::Signal::SIGINT);
    signal::sigprocmask(signal::SigmaskHow::SIG_BLOCK, Some(&sigset), None).unwrap();
    let mut device = Arc::new(Mutex::new(
        ethernet::EthernetDevice::open(args[1].as_str(), raw::Type::Auto).unwrap(),
    ));
    device.run().unwrap();
    loop {
        if signal::Signal::SIGINT == sigset.wait().unwrap() {
            break;
        }
    }

    device.close().unwrap();
}
