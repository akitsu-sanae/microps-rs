
pub struct Datagram {
    pub src_port: u16,
    pub dst_port: u16,
    pub len: u16,
    pub sum: u16,
    pub data: buffer::Buffer,
}


