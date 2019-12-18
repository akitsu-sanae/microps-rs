use crate::{
    buffer,
    ip::{self, interface::Interface},
    util,
};
use nix::errno::{errno, Errno};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Condvar, Mutex};
use uuid::Uuid;

mod packet;
mod queue;

struct Cb {
    interface: Option<Interface>,
    port: u16,
    queue: queue::Queue,
    cond: Condvar,
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
        peer_port: u16,
    ) -> Result<(), Box<dyn Error>> {
        let mut cb_table = CB_TABLE.lock().unwrap();
        let ref mut cb = cb_table.get_mut(&self.id).unwrap();
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
    }

    pub fn recv_from(
        &mut self,
        timeout: i32,
    ) -> Result<(ip::Addr, u16, buffer::Buffer), Box<dyn Error>> {
        let mut cb_table = CB_TABLE.lock().unwrap();
        let cb = cb_table.get_mut(&self.id).unwrap(); // TODO
        while cb.queue.data.is_empty() && errno() == Errno::ETIMEDOUT as i32 {
            if timeout != -1 {
                let _table = cb.cond.wait_timeout(
                    CB_TABLE.lock().unwrap(),
                    ::std::time::Duration::from_secs(timeout as u64),
                )?;
            } else {
                let _table = cb.cond.wait(CB_TABLE.lock().unwrap())?;
            }
        }
        if errno() == Errno::ETIMEDOUT as i32 {
            return Err(util::RuntimeError::new(format!("timeout")));
        }
        let entry = cb.queue.pop().unwrap();
        Ok((entry.addr, entry.port, entry.data))
    }

    pub fn send_to(
        &mut self,
        buf: buffer::Buffer,
        peer_addr: ip::Addr,
        peer_port: u16,
    ) -> Result<(), Box<dyn Error>> {
        let mut cb_table = CB_TABLE.lock().unwrap();
        let ref mut cb = cb_table.get_mut(&self.id).unwrap();
        let interface = cb
            .interface
            .clone()
            .or_else(|| ip::interface::by_addr(peer_addr))
            .unwrap();
        // TODO: do something when `cb.port` is `None`
        tx(&interface, cb.port, buf, peer_addr, peer_port)
    }

    pub fn close(&self) -> Result<(), Box<dyn Error>> {
        let mut cb_table = CB_TABLE.lock().unwrap();
        cb_table.remove(&self.id).unwrap();
        Ok(())
    }
}

pub fn open() -> Result<Socket, Box<dyn Error>> {
    let mut cb_table = CB_TABLE.lock().unwrap();
    let uuid = Uuid::new_v4();
    let cb = Cb {
        interface: None,
        port: 0,
        cond: Condvar::new(),
        queue: queue::Queue::new(),
    };
    cb_table.insert(uuid, cb);
    Ok(Socket { id: uuid })
}

pub fn rx(
    buf: buffer::Buffer,
    src: &ip::Addr,
    _dst: &ip::Addr,
    interface: &Interface,
) -> Result<(), Box<dyn Error>> {
    let pseudo = 0; // TODO
    if util::calc_checksum(buffer::Buffer::empty(), pseudo) != 0 || true {
        // TODO
        return Err(util::RuntimeError::new(format!("incorrent checksum")));
    }

    use crate::packet::Packet;
    let packet = packet::Packet::from_buffer(buf)?;

    if cfg!(debug_assertions) {
        eprintln!(">>> udp_rx <<<");
        packet.dump();
    }

    let mut cb_table = CB_TABLE.lock().unwrap();
    for (_, ref mut cb) in cb_table.iter_mut() {
        if if let Some(ref interface_) = cb.interface {
            Arc::ptr_eq(&interface.0, &interface_.0)
        } else {
            true
        } && cb.port == packet.dst_port
        {
            let queue_header = queue::Entry {
                addr: *src,
                port: packet.src_port,
                data: packet.payload,
            };
            cb.queue.push(queue_header);
            cb.cond.notify_all();
            break;
        }
    }
    Ok(())
}

pub fn tx(
    _interface: &Interface,
    _src_port: u16,
    _buf: buffer::Buffer,
    _peer_addr: ip::Addr,
    _peer_port: u16,
) -> Result<(), Box<dyn Error>> {
    unimplemented!()
}
