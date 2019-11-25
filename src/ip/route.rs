
use std::sync::{Arc, Mutex};

use crate::{frame, ip::Interface};

#[derive(Debug, Clone)]
pub struct Route {
    pub network: frame::IpAddr,
    pub netmask: frame::IpAddr,
    pub nexthop: frame::IpAddr,
    pub interface: Interface,
}

lazy_static! {
    static ref ROUTE_TABLE: Arc<Mutex<Vec<Route>>> = Arc::new(Mutex::new(vec![]));
}

pub fn add(route: Route) {
    let mut route_table = ROUTE_TABLE.lock().unwrap();
    route_table.push(route);
}

pub fn delete(interface: &Interface) {
    let mut route_table = ROUTE_TABLE.lock().unwrap();
    route_table.retain(|route| &route.interface as *const Interface == interface as *const Interface);
}

pub fn lookup(interface: &Interface, dst: frame::IpAddr) -> Option<Route> {
    let route_table = ROUTE_TABLE.lock().unwrap();
    let mut candidate = None;
    for route in route_table.iter() {
        if dst.apply_mask(&route.netmask) == route.network && &route.interface as *const Interface == interface as *const Interface {
            match candidate {
                None =>
                    candidate = Some(route.clone()),
                Some(c) if c.netmask < route.netmask => {
                    candidate = Some(route.clone());
                },
                _ => (),
            }
        }
    }
    candidate
}


