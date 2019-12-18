use crate::{buffer, ip};
use std::collections::VecDeque;

pub struct Entry {
    pub addr: ip::Addr,
    pub port: u16,
    pub data: buffer::Buffer,
}

pub struct Queue {
    pub data: VecDeque<Entry>,
}

impl Queue {
    pub fn new() -> Queue {
        Queue {
            data: VecDeque::new(),
        }
    }

    pub fn push(&mut self, entry: Entry) {
        self.data.push_back(entry);
    }

    pub fn pop(&mut self) -> Option<Entry> {
        self.data.pop_front()
    }
}
