use crate::{buffer::Buffer, ip, util};
use chrono::{DateTime, Utc};
use std::error::Error;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Fragment {
    pub src: ip::Addr,
    pub dst: ip::Addr,
    pub id: u16,
    pub protocol: ip::ProtocolType,
    pub data: Buffer,
    pub mask: Buffer,
    pub timestamp: Option<DateTime<Utc>>,
}

lazy_static! {
    static ref FRAGMENTS: Arc<Mutex<Vec<Fragment>>> = Arc::new(Mutex::new(vec![]));
}

impl Fragment {
    fn new(dgram: &ip::dgram::Dgram) -> Self {
        Fragment {
            src: dgram.src.clone(),
            dst: dgram.dst.clone(),
            id: dgram.id.clone(),
            protocol: dgram.protocol,
            data: Buffer::new(65535),
            mask: Buffer::new(2048),
            timestamp: None,
        }
    }
}

fn patrol() {
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

fn lookup<Pred: Fn(&Fragment) -> bool>(pred: Pred) -> Option<Fragment> {
    let mut fragments = FRAGMENTS.lock().unwrap();
    fragments
        .iter()
        .position(pred)
        .map(|index| fragments.remove(index))
}

lazy_static! {
    static ref PREV_TIME: Arc<Mutex<DateTime<Utc>>> = Arc::new(Mutex::new(Utc::now()));
    static ref FRAGMENT_COUNT: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
}

pub fn process(dgram: ip::dgram::Dgram) -> Result<Fragment, Box<dyn Error>> {
    let mut prev_time = PREV_TIME.lock().unwrap();
    let now = Utc::now();
    if (now - *prev_time as DateTime<Utc>).num_seconds() < 10 {
        patrol();
    }
    *prev_time = now;

    let mut fragment = match lookup(|fragment| {
        fragment.src == dgram.src
            && fragment.dst == dgram.dst
            && fragment.id == dgram.id
            && fragment.protocol == dgram.protocol
    }) {
        Some(fragment) => fragment,
        None => {
            const NUM_MAX: i32 = 8;
            let mut count = FRAGMENT_COUNT.lock().unwrap();
            if *count >= NUM_MAX {
                return Err(util::RuntimeError::new(format!("too many fragments")));
            }
            let fragment = Fragment::new(&dgram);
            *count += 1;
            fragment
        }
    };

    let off = ((dgram.offset & 0x1fff) << 3) as usize;
    let payload_len = dgram.payload.0.len();
    fragment.data.write(off, dgram.payload);
    // TODO: set mask data

    fragment.timestamp = Some(Utc::now());
    if dgram.offset & 0x2000 != 0 {
        let mut fragments = FRAGMENTS.lock().unwrap();
        fragments.push(fragment);
        return Err(util::RuntimeError::new(format!("more fragments exists")));
    }
    fragment.data.0.resize(off + payload_len, 0);

    if !check_mask(&fragment.mask, fragment.data.0.len()) {
        return Err(util::RuntimeError::new(format!("imcomplete flagments")));
    }
    let mut count = FRAGMENT_COUNT.lock().unwrap();
    *count -= 1;
    Ok(fragment)
}

fn check_mask(_mask: &Buffer, _data_len: usize) -> bool {
    true // TODO: check!!
}
