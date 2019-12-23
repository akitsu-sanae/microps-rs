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
    pub mask: Vec<u32>,
    pub timestamp: Option<DateTime<Utc>>,
}

lazy_static! {
    static ref FRAGMENTS: Arc<Mutex<Vec<Fragment>>> = Arc::new(Mutex::new(vec![]));
}

impl Fragment {
    fn new(dgram: &ip::dgram::Dgram) -> Self {
        let mut mask = vec![];
        mask.resize(2048, 0);
        Fragment {
            src: dgram.src.clone(),
            dst: dgram.dst.clone(),
            id: dgram.id.clone(),
            protocol: dgram.protocol,
            data: Buffer::new(65535),
            mask: mask,
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
    set_mask(&mut fragment.mask, off, payload_len);

    fragment.timestamp = Some(Utc::now());
    if dgram.offset & 0x2000 != 0 {
        let mut fragments = FRAGMENTS.lock().unwrap();
        fragments.push(fragment);
        return Err(util::RuntimeError::new(format!("more fragments exists")));
    }
    fragment.data.0.resize(off + payload_len, 0);

    if !check_mask(&fragment.mask, 0, fragment.data.0.len()) {
        return Err(util::RuntimeError::new(format!("imcomplete flagments")));
    }
    let mut count = FRAGMENT_COUNT.lock().unwrap();
    *count -= 1;
    Ok(fragment)
}

fn set_mask(mask: &mut Vec<u32>, offset: usize, mut len: usize) {
    let so = offset / 32;
    let sb = offset % 32;
    let bl = if len > 32 - sb {
        32 - sb
    } else {
        len
    };
    mask[so] |= (0xffffffff >> (32 - bl)) << sb;
    len -= bl;
    for idx in so .. so+len/32 {
        mask[idx + 1] = 0xffffffff;
    }
    let i = so+len/32;
    len -= 32 * (len/32);
    if len != 0 {
        mask[i+1] |= 0xffffffff >> (32 - len);
    }
}

fn check_mask(mask: &Vec<u32>, offset: usize, mut data_len: usize) -> bool {
    let so = offset / 32;
    let sb = offset % 32;
    let bl = if data_len > 32 - sb {
        32 - sb
    } else {
        data_len
    };
    if (mask[offset / 32] & ((0xffffffff >> (32 - bl)) << sb)) ^ ((0xffffffff >> (32 - bl)) << sb) != 0 {
        return false;
    }
    data_len -= bl;
    for idx in so .. so+data_len/32 {
        if mask[idx + 1] ^ 0xffffffff != 0 {
            return false;
        }
    }
    let i = so+data_len/32;
    data_len -= 32 * (data_len/32);
    if data_len != 0 {
        if (mask[i + 1] & (0xffffffff >> (32 - data_len))) ^ (0xffffffff >> (32 - data_len)) != 0 {
            return false;
        }
    }
    true
}


