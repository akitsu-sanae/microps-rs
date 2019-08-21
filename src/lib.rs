use std::error::Error;

pub mod tcp_api;

pub struct Microps {}

impl Microps {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        unimplemented!()
    }
}

impl Drop for Microps {
    fn drop(&mut self) {
        unimplemented!()
    }
}
