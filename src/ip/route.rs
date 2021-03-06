use std::sync::{Arc, Mutex};

use crate::ip::{self, interface::Interface};

#[derive(Debug, Clone)]
pub struct Route {
    pub network: ip::Addr,
    pub netmask: ip::Addr,
    pub nexthop: Option<ip::Addr>,
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
    route_table
        .retain(|route| &route.interface as *const Interface == interface as *const Interface);
}

pub fn lookup(interface: Option<&Interface>, dst: ip::Addr) -> Option<Route> {
    let route_table = ROUTE_TABLE.lock().unwrap();
    let mut candidate = None;
    for route in route_table.iter() {
        let is_same_interface = if let Some(interface) = interface {
            &route.interface as *const Interface == interface as *const Interface
        } else {
            true
        };
        if dst.apply_mask(&route.netmask) == route.network && is_same_interface {
            match candidate {
                None => candidate = Some(route.clone()),
                Some(c) if c.netmask < route.netmask => {
                    candidate = Some(route.clone());
                }
                _ => (),
            }
        }
    }
    candidate
}
