use crate::{
    buffer,
    ip::{self, interface::Interface},
    protocol, util,
};
use nix::errno::{errno, Errno};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Condvar, Mutex, RwLock};
use uuid::Uuid;

mod packet;
mod queue;

const SOURCE_PORT_MIN: u16 = 49152;
const SOURCE_PORT_MAX: u16 = 65535;

struct Cb {
    interface: Option<Interface>,
    port: u16,
    queue: queue::Queue,
}

lazy_static! {
    static ref CB_TABLE: Arc<Mutex<HashMap<Uuid, Cb>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref CONDS_PUSHED: Arc<RwLock<HashMap<Uuid, Condvar>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

pub struct Socket {
    id: Uuid,
}

impl Socket {
    pub fn bind(
        &mut self,
        peer_addr: ip::Addr,
        peer_port: u16,
    ) -> Result<(), Box<dyn Error>> {
        let mut cb_table = CB_TABLE.lock().unwrap();
        let ref mut cb = cb_table.get_mut(&self.id).unwrap();
        let interface = match ip::interface::by_addr(peer_addr) {
                Some(interface) => interface,
                None => return Err(util::RuntimeError::new(format!("invalid addr: {}", peer_addr))),
        };
        cb.interface = Some(interface);
        cb.port = peer_port;
        Ok(())
    }
    pub fn bind_interface(
        &mut self,
        interface: Interface,
        peer_port: u16,
    ) -> Result<(), Box<dyn Error>> {
        let mut cb_table = CB_TABLE.lock().unwrap();
        let ref mut cb = cb_table.get_mut(&self.id).unwrap();
        cb.interface = Some(interface);
        cb.port = peer_port;
        Ok(())
    }

    pub fn recv_from(
        &mut self,
        timeout: i32,
    ) -> Result<(ip::Addr, u16, buffer::Buffer), Box<dyn Error>> {
        loop {
            if timeout != -1 {
                let conds_pushed = CONDS_PUSHED.read().unwrap();
                let ref cond_pushed = conds_pushed.get(&self.id).unwrap();
                let cb_table = CB_TABLE.lock().unwrap();
                cond_pushed
                    .wait_timeout(cb_table, ::std::time::Duration::from_secs(timeout as u64))?;
            } else {
                let conds_pushed = CONDS_PUSHED.read().unwrap();
                let ref cond_pushed = conds_pushed.get(&self.id).unwrap();
                let cb_table = CB_TABLE.lock().unwrap();
                cond_pushed.wait(cb_table);
            };

            if errno() == Errno::ETIMEDOUT as i32 {
                return Err(util::RuntimeError::new(format!("timeout")));
            }

            let mut cb_table = CB_TABLE.lock().unwrap();
            let ref mut cb = cb_table.get_mut(&self.id).unwrap();
            if let Some(entry) = cb.queue.pop() {
                return Ok((entry.addr, entry.port, entry.data));
            }
        }
    }

    pub fn send_to(
        &mut self,
        buf: buffer::Buffer,
        peer_addr: ip::Addr,
        peer_port: u16,
    ) -> Result<(), Box<dyn Error>> {
        let mut cb_table = CB_TABLE.lock().unwrap();
        let ref cb = cb_table.get_mut(&self.id).unwrap();
        let interface = cb
            .interface
            .clone()
            .or_else(|| ip::interface::by_addr(peer_addr))
            .unwrap();

        let port = if cb.port == 0 {
            let mut result = None;
            for port in { SOURCE_PORT_MIN..SOURCE_PORT_MAX } {
                let is_found = cb_table.iter().any(|(_, ref cb)| {
                    cb.port == port
                        && cb
                            .interface
                            .as_ref()
                            .map(|interface_| Arc::ptr_eq(&interface.0, &interface_.0))
                            .unwrap_or(true)
                });
                if is_found {
                    result = Some(port);
                    break;
                }
            }
            result
        } else {
            Some(cb.port)
        };

        let ref mut cb = cb_table.get_mut(&self.id).unwrap();
        if let Some(port) = port {
            cb.port = port;
        } else {
            return Err(util::RuntimeError::new(format!("not found : valid port")));
        }
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
        queue: queue::Queue::new(),
    };
    cb_table.insert(uuid, cb);

    let mut conds_pushed = CONDS_PUSHED.write().unwrap();
    conds_pushed.insert(uuid, Condvar::new());

    Ok(Socket { id: uuid })
}

pub fn rx(
    buf: buffer::Buffer,
    src: &ip::Addr,
    _dst: &ip::Addr,
    interface: &Interface,
) -> Result<(), Box<dyn Error>> {
    let pseudo = 0; // TODO
    if util::calc_checksum(buffer::Buffer::empty(), pseudo) != 0 && false {
        // TODO
        return Err(util::RuntimeError::new(format!("incorrect checksum")));
    }

    use crate::packet::Packet;
    let packet = packet::Packet::from_buffer(buf)?;

    if cfg!(debug_assertions) {
        eprintln!(">>> udp_rx <<<");
        packet.dump();
    }

    let mut cb_table = CB_TABLE.lock().unwrap();
    for (ref id, ref mut cb) in cb_table.iter_mut() {
        let is_same_interface = cb
            .interface
            .as_ref()
            .map(|interface_| Arc::ptr_eq(&interface.0, &interface_.0))
            .unwrap_or(true);
        if is_same_interface && cb.port == packet.dst_port {
            let queue_header = queue::Entry {
                addr: *src,
                port: packet.src_port,
                data: packet.payload,
            };
            cb.queue.push(queue_header);

            let conds_pushed = CONDS_PUSHED.read().unwrap();
            let cond = conds_pushed.get(id).unwrap();
            cond.notify_all();
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

pub struct UdpProtocol {}

impl UdpProtocol {
    pub fn new() -> Arc<dyn protocol::Protocol + Send + Sync> {
        Arc::new(UdpProtocol {})
    }
}

impl protocol::Protocol for UdpProtocol {
    fn type_(&self) -> protocol::ProtocolType {
        protocol::ProtocolType::Udp
    }
    fn handler(
        &self,
        payload: buffer::Buffer,
        src: ip::Addr,
        dst: ip::Addr,
        interface: &Interface,
    ) -> Result<(), Box<dyn Error>> {
        self::rx(payload, &src, &dst, interface)
    }
}
