use std::sync::{Arc, Mutex};
use std::error::Error;
use chrono::{DateTime, Utc};
use crate::{ip, util, frame};

#[derive(Debug)]
pub struct Fragment {
    pub src: frame::IpAddr,
    pub dst: frame::IpAddr,
    pub id: u16,
    pub protocol: ip::ProtocolType,
    pub len: u16,
    pub data: frame::Bytes,
    pub mask: frame::Bytes,
    pub timestamp: Option<DateTime<Utc>>,
}

lazy_static! {
    static ref FRAGMENTS: Arc<Mutex<Vec<Fragment>>> = Arc::new(Mutex::new(vec![]));
}

impl Fragment {
    pub fn new(dgram: &ip::Dgram) -> Self {
        Fragment {
            src: dgram.src.clone(),
            dst: dgram.dst.clone(),
            id: dgram.id.clone(),
            protocol: dgram.protocol,
            len: 0,
            data: frame::Bytes::empty(),
            mask: frame::Bytes::empty(),
            timestamp: None,
        }
    }

    pub fn detach(&self) -> Result<Self, Box<dyn Error>> {
        let mut fragments = FRAGMENTS.lock().unwrap();
        match fragments.iter().position(|fragment| fragment as *const Fragment == self as *const Fragment) {
            None => Err(util::RuntimeError::new(format!("can not detach unregistered fragment! {:?}", self))),
            Some(index) =>
                Ok(fragments.remove(index))
        }
    }
}

pub fn patrol() {
    let now = Utc::now();
    let mut fragments = FRAGMENTS.lock().unwrap();
    const TIMEOUT_SEC: i64 = 30;

    fragments.retain(|fragment| {
        if let Some(timestamp) = fragment.timestamp {
            (now - timestamp).num_seconds() < TIMEOUT_SEC
        } else {
            false
        }
    });
}

