extern crate microps_rs;

use std::collections::VecDeque;
use microps_rs::{udp, ethernet::MacAddr, ip};

fn print_usage(name: &str) {
    eprintln!("usage: {} <interface> [mac_addr] static <ip_addr> <netmask> [gateway]", name);
    eprintln!("   or: {} <interface> [mac_addr] dhcp", name);
}

enum Args {
    Static {
        interface: String,
        mac_addr: Option<MacAddr>,
        ip_addr: ip::Addr,
        netmask: ip::Addr,
        gateway: Option<ip::Addr>,
    },
    Dhcp {
        interface: String,
        mac_addr: Option<MacAddr>,
    },
}

fn parse_args() -> Args {
    let mut args: VecDeque<String> = ::std::env::args().into_iter().collect();
    let program_name = args.pop_front().unwrap();

    match (|| {
        let interface = args.pop_front()?;
        let mac_addr = if args[0] == "static" || args[0] == "dhcp" {
            None
        } else {
            Some(MacAddr::from_str(&args.pop_front()?).unwrap())
        };
        match args.pop_front()?.as_str() {
            "static" => {
                let ip_addr = ip::Addr::from_str(&args.pop_front()?).unwrap();
                let netmask = ip::Addr::from_str(&args.pop_front()?).unwrap();
                let gateway = if args.is_empty() {
                    None
                } else {
                    let gateway = ip::Addr::from_str(&args.pop_front()?).unwrap();
                    Some(gateway)
                };
                Some(Args::Static {
                    interface: interface,
                    mac_addr: mac_addr,
                    ip_addr: ip_addr,
                    netmask: netmask,
                    gateway: gateway,
                })
            },
            "dhcp" => {
                Some(Args::Dhcp {
                    interface: interface,
                    mac_addr: mac_addr,
                })
            },
            _ => {
                print_usage(&program_name);
                panic!()
            }
        }

    })() {
        Some(args) => args,
        None => {
            print_usage(&program_name);
            panic!()
        }
    }
}

fn main() {
    let mut socket = udp::open().unwrap();
    socket.bind(None, 7).unwrap();
    eprintln!("waiting for message...");
    loop {
        let (peer_addr, peer_port, buf) = socket.recv_from(-1).unwrap();
        if buf.0.is_empty() {
            break;
        }
        eprintln!("message from: {}:{}", peer_addr, peer_port);
        eprintln!("{}", buf);
        socket.send_to(buf, peer_addr, peer_port).unwrap();
    }
    socket.close().unwrap();
}

