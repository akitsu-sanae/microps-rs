use std::error::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    pub message: String,
}

impl RuntimeError {
    pub fn new(message: String) -> Box<dyn Error> {
        Box::new(RuntimeError { message: message })
    }
}

use std::fmt;
impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "runtime error : {}", self.message)
    }
}

impl Error for RuntimeError {
    fn description(&self) -> &str {
        self.message.as_str()
    }
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

pub fn htons(n: u16) -> u16 {
    n.to_be()
}
pub fn ntohs(n: u16) -> u16 {
    u16::from_be(n)
}

pub fn calc_checksum(data: &[u8], len: usize, init: u32) -> u16 {
    use std::slice;
    let data_u16: &[u16] = unsafe { slice::from_raw_parts(data.as_ptr() as *const u16, len / 2) };
    let mut index = 0;

    let mut sum = init;
    while index + 1 < len {
        sum += data_u16[index / 2] as u32;
        index += 2;
    }
    if index + 1 != len {
        sum += data[index - 1] as u32;
    }
    sum = (sum & 0xffff) + (sum >> 16);
    sum = (sum & 0xffff) + (sum >> 16);
    return !(sum as u16);
}
