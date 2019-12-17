use std::collections::VecDeque;
use crate::buffer::Buffer;

pub struct Queue {
    data: VecDeque<Buffer>,
}

impl Queue {
    pub fn new() -> Queue {
        Queue {
            data: VecDeque::new(),
        }
    }
    pub fn push(&mut self, data: Buffer) {
        self.data.push_back(data);
    }
    pub fn pop(&mut self) -> Option<Buffer> {
        self.data.pop_front()
    }
}

