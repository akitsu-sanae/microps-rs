use crate::buffer::Buffer;
use std::error::Error;

pub trait Packet<T> {
    fn from_buffer(buffer: Buffer) -> Result<T, Box<dyn Error>>;
    fn to_buffer(self) -> Buffer;
}
