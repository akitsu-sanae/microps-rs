extern crate microps_rs;
extern crate ctrlc;

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use microps_rs::raw::tap;

fn main() {
    let terminate = Arc::new(AtomicBool::new(false));
    let t = terminate.clone();

    ctrlc::set_handler(move || {
        t.store(true, Ordering::SeqCst);
    }).expect("failed: set Ctrl-C handler");

    let args: Vec<String> = ::std::env::args().into_iter().collect();
    if args.len() != 2 {
        panic!("USAGE: raw_tap_test <device>");
    }
    let mut device = tap::TapDevice::open(args[1].as_str()).unwrap();
    eprintln!("[{}] {:?}", device.name(), device.addr());

    while !terminate.load(Ordering::SeqCst) {
        device.rx(|_frame: &Vec<u8>, len: usize, _arg: &Vec<u8>| {
            println!("receive {} octets", len);
        }, &vec![], 1000);
    }
    device.close();
}
