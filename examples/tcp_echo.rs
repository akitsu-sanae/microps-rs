extern crate microps_rs;

use microps_rs::{ethernet, net};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Interface(String);
#[derive(Debug, Clone, PartialEq, Eq)]
struct Hwaddr(String);
#[derive(Debug, Clone, PartialEq, Eq)]
struct Ipaddr(String);
#[derive(Debug, Clone, PartialEq, Eq)]
struct Netmask(String);
#[derive(Debug, Clone, PartialEq, Eq)]
struct Gateway(String);

#[derive(Debug, Clone, PartialEq, Eq)]
enum IpaddrArgs {
    Dhcp,
    Static(Ipaddr, Netmask, Option<Gateway>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    interface: Interface,
    hwaddr: Option<Hwaddr>,
    ipaddr_arg: IpaddrArgs,
}

const USAGE: &str = r#"
USAGE: <TODO>
"#;

fn parse_args() -> Args {
    let mut args: Vec<String> = ::std::env::args().into_iter().rev().collect();
    let _prog_name = args.pop().expect(USAGE);
    let interface = Interface(args.pop().expect(USAGE));
    let hwaddr = match args.last().expect(USAGE).clone().as_str() {
        "static" | "dhcp" => None,
        hwaddr => {
            args.pop().unwrap();
            Some(Hwaddr(hwaddr.to_string()))
        }
    };
    let ipaddr_arg = match args.pop().expect(USAGE).as_str() {
        "static" => {
            let ipaddr = Ipaddr(args.pop().expect(USAGE));
            let netmask = Netmask(args.pop().expect(USAGE));
            let gateway = args.pop().map(|gateway| Gateway(gateway));
            IpaddrArgs::Static(ipaddr, netmask, gateway)
        }
        "dhcp" => IpaddrArgs::Dhcp,
        _ => panic!(USAGE),
    };
    Args {
        interface: interface,
        hwaddr: hwaddr,
        ipaddr_arg: ipaddr_arg,
    }
}

fn main() {
    let args = parse_args();
    println!("{:?}", args);
    let microps = microps_rs::Microps::new().unwrap();
    let mut dev = microps.net.alloc(net::Type::Ethernet).unwrap();
    dev.name = args.interface;
    if let Some(Hwaddr(hwaddr)) = args.hwaddr {
        ethernet::addr_pton(hwaddr, dev.addr);
    }
    // TODO
    unimplemented!()
}
