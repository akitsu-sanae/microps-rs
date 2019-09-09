use crate::net;

pub trait Interface {
    fn family(&self) -> net::Interface;
}

