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

pub fn hton16(h: usize) -> usize {
    unimplemented!()
}
