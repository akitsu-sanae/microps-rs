#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate microps_rs;
extern crate nix;

use microps_rs::{raw::socket, util::hexdump};
use nix::sys::signal::{self, SigHandler, Signal};
use std::sync::atomic::{AtomicBool, Ordering};

lazy_static! {
    static ref TERMINATE: AtomicBool = AtomicBool::new(false);
}

extern "C" fn handle_sigint(signal: libc::c_int) {
    let signal = Signal::from_c_int(signal).unwrap();
    TERMINATE.store(signal == Signal::SIGINT, Ordering::Relaxed);
}

fn dump(frame: &Vec<u8>, name: String) {
    eprintln!("{}: receive {} octets", name, frame.len());
    hexdump(frame);
}

fn main() {
    let args: Vec<String> = ::std::env::args().into_iter().collect();
    if args.len() != 2 {
        panic!("USAGE: raw_socket_test <device>");
    }
    let handler = SigHandler::Handler(handle_sigint);
    unsafe { signal::signal(Signal::SIGINT, handler) }.unwrap();

    let device = socket::SocketDevice::open(args[1].as_str()).unwrap();
    let mut device = device.lock().unwrap();
    eprintln!("[{}] {}", device.name(), device.addr().unwrap());

    while !TERMINATE.load(Ordering::Relaxed) {
        let name = device.name().clone();
        device.rx(Box::new(|frame: &Vec<u8>| dump(frame, name)), 1000);
    }
    device.close().unwrap();
}
