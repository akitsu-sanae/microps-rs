extern crate ctrlc;
extern crate microps_rs;

use microps_rs::raw::socket;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

fn main() {
    let terminate = Arc::new(AtomicBool::new(false));
    let t = terminate.clone();

    ctrlc::set_handler(move || {
        t.store(true, Ordering::SeqCst);
    })
    .expect("failed: set Ctrl-C handler");

    let args: Vec<String> = ::std::env::args().into_iter().collect();
    if args.len() != 2 {
        panic!("USAGE: raw_socket_test <device>");
    }
    let mut device = socket::SocketDevice::open(args[1].as_str()).unwrap();
    eprintln!("[{}] {:?}", device.name(), device.addr());

    while !terminate.load(Ordering::SeqCst) {
        device.rx(
            |_frame: &Vec<u8>, len: usize, _arg: &Vec<u8>| {
                println!("receive {} octets", len);
            },
            &vec![],
            1000,
        );
    }
    device.close();
}