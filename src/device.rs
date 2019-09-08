use std::error::Error;
use std::sync::{Arc, Mutex};

use crate::interface::Interface;

pub trait Device {
    fn name(&self) -> &String;
    fn regist_interface(&mut self, interface: Arc<Mutex<dyn Interface>>);
    fn run(device: Arc<Mutex<Self>>) -> Result<(), Box<dyn Error>>;
    fn close(&mut self) -> Result<(), Box<dyn Error>>;
}

