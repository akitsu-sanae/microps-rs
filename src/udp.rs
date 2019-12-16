use crate::{
    buffer,
    ip::{self, interface::Interface},
    util,
};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

struct Cb {
    interface: Option<Interface>,
    port: i32,
}

lazy_static! {
    static ref CB_TABLE: Arc<Mutex<HashMap<Uuid, Cb>>> = Arc::new(Mutex::new(HashMap::new()));
}

pub struct Socket {
    id: Uuid,
}

impl Socket {
    pub fn bind(
        &mut self,
        peer_addr: Option<ip::Addr>,
        peer_port: i32,
    ) -> Result<(), Box<dyn Error>> {
        let mut cb_table = CB_TABLE.lock().unwrap();
        if let Some(ref mut cb) = cb_table.get_mut(&self.id) {
            let interface = if let Some(addr) = peer_addr {
                match ip::interface::by_addr(addr) {
                    Some(interface) => Some(interface),
                    None => return Err(util::RuntimeError::new(format!("invalid addr: {}", addr))),
                }
            } else {
                None
            };
            cb.interface = interface;
            cb.port = peer_port;
            Ok(())
        } else {
            Err(util::RuntimeError::new(format!(
                "invalid uuid: {}",
                self.id
            )))
        }
    }

    pub fn recv_from(
        &mut self,
        _timeout: i32,
    ) -> Result<(ip::Addr, i32, buffer::Buffer), Box<dyn Error>> {
        unimplemented!()
    }

    pub fn send_to(
        &mut self,
        _buf: buffer::Buffer,
        _peer_addr: ip::Addr,
        _peer_port: i32,
    ) -> Result<(), Box<dyn Error>> {
        unimplemented!()
    }

    pub fn close(&self) -> Result<(), Box<dyn Error>> {
        unimplemented!()
    }
}

pub fn open() -> Result<Socket, Box<dyn Error>> {
    let mut cb_table = CB_TABLE.lock().unwrap();
    let uuid = Uuid::new_v4();
    let cb = Cb {
        interface: None,
        port: 0,
    };
    cb_table.insert(uuid, cb);
    Ok(Socket { id: uuid })
}
