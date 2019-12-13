use std::error::Error;
use std::sync::{Arc, Condvar, Mutex};

use chrono::{DateTime, Duration, Utc};

use crate::{buffer::Buffer, ethernet, ip, util::RuntimeError};

#[derive(Debug)]
pub struct Entry {
    pub ip_addr: ip::Addr,
    pub mac_addr: ethernet::MacAddr,
    pub timestamp: DateTime<Utc>,
    pub cond: Condvar,
    pub data: Buffer,
    pub interface: ip::interface::Interface,
}

impl Entry {
    pub fn new(
        ip_addr: ip::Addr,
        mac_addr: ethernet::MacAddr,
        interface: ip::interface::Interface,
    ) -> Entry {
        Entry {
            ip_addr: ip_addr,
            mac_addr: mac_addr,
            timestamp: Utc::now(),
            cond: Condvar::new(),
            data: Buffer::empty(),
            interface: interface,
        }
    }
}

lazy_static! {
    pub static ref TABLE: Arc<Mutex<Vec<Entry>>> = Arc::new(Mutex::new(vec![]));
    static ref TIMESTAMP: Mutex<DateTime<Utc>> = Mutex::new(Utc::now());
}

pub fn lookup(ip_addr: &ip::Addr) -> Option<usize> {
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
        Err(RuntimeError::new(format!("out of range: index is {}, but table size is {}", idx, table.len())))
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
