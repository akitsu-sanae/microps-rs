use std::error::Error;
use std::sync::{Arc, Mutex};

use crate::{interface::Interface, frame, ethernet};

pub trait Device {
    fn name(&self) -> String;
    fn add_interface(&mut self, interface: Arc<Mutex<dyn Interface + Send>>) -> Result<(), Box<dyn Error>>;
    fn run(&mut self) -> Result<(), Box<dyn Error>>;
    fn close(self) -> Result<(), Box<dyn Error>>;
    fn tx(&mut self, type_: ethernet::Type,  packet: Vec<u8>, dst: &frame::MacAddr);
}
