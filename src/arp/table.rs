use std::error::Error;
use std::sync::{Arc, Condvar, Mutex};

use chrono::{DateTime, Duration, Utc};

use crate::{frame, ipv4, util::RuntimeError};

#[derive(Debug)]
pub struct Entry {
    pub ip_addr: frame::Ipv4Addr,
    pub mac_addr: frame::MacAddr,
    pub timestamp: DateTime<Utc>,
    pub cond: Condvar,
    pub data: frame::Bytes,
    pub interface: ipv4::Interface,
}

impl Entry {
    pub fn new(
        ip_addr: frame::Ipv4Addr,
        mac_addr: frame::MacAddr,
        interface: ipv4::Interface,
    ) -> Entry {
        Entry {
            ip_addr: ip_addr,
            mac_addr: mac_addr,
            timestamp: Utc::now(),
            cond: Condvar::new(),
            data: frame::Bytes::empty(),
            interface: interface,
        }
    }
}

lazy_static! {
    pub static ref TABLE: Arc<Mutex<Vec<Entry>>> = Arc::new(Mutex::new(vec![]));
    static ref TIMESTAMP: Mutex<DateTime<Utc>> = Mutex::new(Utc::now());
}

pub fn lookup(ip_addr: &frame::Ipv4Addr) -> Option<usize> {
    let table = TABLE.lock().unwrap();
    for (idx, entry) in table.iter().enumerate() {
        if &entry.ip_addr == ip_addr {
            return Some(idx);
        }
    }
    None
}

pub fn remove(idx: usize) -> Result<(), Box<dyn Error>> {
    let mut table = TABLE.lock().unwrap();
    if idx < table.len() {
        table.remove(idx);
        Ok(())
    } else {
        Err(RuntimeError::new("".to_string())) // TODO
    }
}

pub fn patrol() {
    let mut timestamp = TIMESTAMP.lock().unwrap();
    if (Utc::now() - *timestamp).num_seconds() > 10 {
        let mut table = TABLE.lock().unwrap();
        *timestamp = Utc::now();
        let _: Vec<_> = table
            .drain_filter(|entry| {
                let timeout = Duration::seconds(300);
                if *timestamp - entry.timestamp > timeout {
                    entry.cond.notify_all();
                    true
                } else {
                    false
                }
            })
            .collect();
    }
}
