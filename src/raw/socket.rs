use super::{RawDevice, Type};
use crate::ethernet::ADDR_LEN;
use crate::frame::{Bytes, MacAddr};
use crate::util::*;
use ifstructs::ifreq;
use libc::{self, pollfd, ETH_P_ALL, POLLIN};
use nix::{
    errno::{errno, Errno},
    sys::socket::{bind, socket, AddressFamily, LinkAddr, SockAddr, SockFlag, SockType},
    unistd,
};
use std::convert::TryInto;
use std::error::Error;
use std::sync::{Arc, Mutex};

ioctl_readwrite_bad!(get_iface_index, 0x8933, ifreq);
ioctl_readwrite_bad!(get_iface_flags, libc::SIOCGIFFLAGS, ifreq);
ioctl_readwrite_bad!(get_hwaddr, libc::SIOCGIFHWADDR, ifreq);

#[derive(Debug)]
pub struct Device {
    fd: i32,
    name: String,
}

impl Device {
    pub fn open(name: &str) -> Result<Arc<Mutex<dyn RawDevice + Send>>, Box<dyn Error>> {
        let mut device = Device {
            fd: socket(
                AddressFamily::Packet,
                SockType::Raw,
                SockFlag::empty(),
                Some(unsafe { ::std::mem::transmute(libc::ETH_P_ALL) }),
            )?,
            name: name.to_string(),
        };
        if device.fd == -1 {
            device.close()?;
            return Err(RuntimeError::new("socket failed".to_string()));
        }
        let mut ifr = ifreq::from_name(name)?;
        if let Err(err) = unsafe { get_iface_index(device.fd, &mut ifr) } {
            device.close()?;
            return Err(Box::new(err));
            // return Err(Box::new(RuntimeError::new("ioctl [SIOCGIFINDEX]".to_string())));
        }
        let socket_addr = SockAddr::Link(LinkAddr(libc::sockaddr_ll {
            sll_family: libc::AF_PACKET.try_into().unwrap(),
            sll_protocol: htons(ETH_P_ALL.try_into().unwrap()),
            sll_ifindex: unsafe { ifr.ifr_ifru.ifr_ifindex },
            sll_hatype: 0,
            sll_pkttype: 0,
            sll_halen: 0,
            sll_addr: [0; 8],
        }));
        if let Err(err) = bind(device.fd, &socket_addr) {
            device.close()?;
            return Err(Box::new(err));
        }
        if let Err(err) = unsafe { get_iface_flags(device.fd, &mut ifr) } {
            device.close()?;
            return Err(Box::new(err));
        }
        unsafe {
            ifr.ifr_ifru.ifr_flags = ifr.ifr_ifru.ifr_flags | (libc::IFF_PROMISC as i16);
        }
        if let Err(err) = unsafe { get_iface_flags(device.fd, &mut ifr) } {
            device.close()?;
            return Err(Box::new(err));
        }
        Ok(Arc::new(Mutex::new(device)))
    }
}

impl RawDevice for Device {
    fn type_(&self) -> Type {
        Type::Socket
    }
    fn name(&self) -> &String {
        &self.name
    }
    fn addr(&self) -> Result<MacAddr, Box<dyn Error>> {
        let fd = socket(
            AddressFamily::Inet,
            SockType::Datagram,
            SockFlag::empty(),
            None,
        )?;
        let mut ifr = ifreq::from_name(self.name.as_str())?;
        ifr.ifr_ifru.ifr_addr.sa_family = libc::AF_INET.try_into().unwrap();
        if let Err(err) = unsafe { get_hwaddr(fd, &mut ifr) } {
            unistd::close(fd)?;
            Err(Box::new(err))
        } else {
            let addr = unsafe { ifr.ifr_ifru.ifr_hwaddr.sa_data };
            let addr =
                unsafe { &*(addr.as_ptr() as *const [i8; ADDR_LEN] as *const [u8; ADDR_LEN]) };
            unsafe {
                libc::close(fd);
            }
            Ok(MacAddr(*addr))
        }
    }
    fn close(&mut self) -> Result<(), Box<dyn Error>> {
        if self.fd != -1 {
            unistd::close(self.fd)? // TODO
        }
        // free device
        Ok(())
    }

    fn rx(
        &self,
        callback: Box<dyn FnOnce(Bytes) -> Result<(), Box<dyn Error>>>,
        timeout: i32,
    ) -> Result<(), Box<dyn Error>> {
        let mut pfd = pollfd {
            fd: self.fd,
            events: POLLIN,
            revents: 0,
        };
        match unsafe { libc::poll(&mut pfd, 1, timeout) } {
            0 => return Ok(()), // timeout
            -1 => {
                if errno() != Errno::EINTR as i32 {
                    return Err(RuntimeError::new("poll error".to_string()));
                } else {
                    return Ok(());
                }
            }
            _ => (),
        }
        let mut buf = vec![];
        buf.resize(2048, 0);
        let len: usize = match unsafe {
            libc::read(self.fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len())
        } {
            0 => return Ok(()), // timeout
            -1 => return Err(RuntimeError::new("read error".to_string())),
            len => len,
        }
        .try_into()
        .unwrap();
        buf.resize(len, 0);
        callback(Bytes::from_vec(buf))
    }

    fn tx(&self, _buf: Bytes) -> Result<(), Box<dyn Error>> {
        unimplemented!()
    }
}
