
pub struct Queue {
    pub addr: ip::Addr,
    pub port: u16,
    pub len: u16,
    pub data: buffer::Buffer,
}

