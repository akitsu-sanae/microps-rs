#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate microps_rs;
extern crate nix;

use nix::sys::signal::{self, SigHandler, Signal};
use std::sync::atomic::{AtomicBool, Ordering};
use microps_rs::{ethernet, raw};

lazy_static! {
    static ref TERMINATE: AtomicBool = AtomicBool::new(false);
}

extern "C" fn handle_sigint(signal: libc::c_int) {
    let signal = Signal::from_c_int(signal).unwrap();
    TERMINATE.store(signal == Signal::SIGINT, Ordering::Relaxed);
}

fn main() {
    let args: Vec<String> = ::std::env::args().into_iter().collect();
    if args.len() != 2 {
        panic!("USAGE: ethernet_test <device>");
    }

    let handler = SigHandler::Handler(handle_sigint);
    unsafe { signal::signal(Signal::SIGINT, handler) }.unwrap();

    let device = ethernet::Device::open(args[1].as_str(), raw::Type::Auto).unwrap();
    device.run().unwrap();
    while !TERMINATE.load(Ordering::SeqCst) {}
    device.close().unwrap();
}
