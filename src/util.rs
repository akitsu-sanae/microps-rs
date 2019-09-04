use std::error::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    pub message: String,
}

impl RuntimeError {
    pub fn new(message: String) -> Self {
        RuntimeError { message: message }
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

pub fn hexdump(data: &Vec<u8>) {
    hexdump::hexdump(data.as_slice());
}
