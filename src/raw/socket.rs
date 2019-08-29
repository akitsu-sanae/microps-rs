use super::{RawDevice, Type};
use crate::ethernet::ADDR_LEN;
use crate::util::*;
use ifstructs::ifreq;
use libc::{self, pollfd, ETH_P_ALL, POLLIN};
use nix::{
    sys::socket::{
        bind, socket, AddressFamily, LinkAddr, SockAddr, SockFlag, SockProtocol, SockType,
    },
    unistd,
};
use std::convert::TryInto;
use std::error::Error;

ioctl_readwrite_bad!(get_iface_index, 0x8933, ifreq);
ioctl_readwrite_bad!(get_iface_flags, libc::SIOCGIFFLAGS, ifreq);
ioctl_readwrite_bad!(get_hwaddr, libc::SIOCGIFHWADDR, ifreq);

pub struct SocketDevice {
    fd: i32,
    name: String,
}

impl SocketDevice {
    pub fn open(name: &str) -> Result<Box<dyn RawDevice>, Box<dyn Error>> {
        use std::convert::TryInto;
        let mut device = SocketDevice {
            fd: socket(
                AddressFamily::Packet,
                SockType::Raw,
                SockFlag::empty(),
                Some(unsafe { ::std::mem::transmute(libc::ETH_P_ALL) }),
            )?,
            name: name.to_string(),
        };
        if device.fd == -1 {
            device.close();
            return Err(Box::new(RuntimeError::new("socket failed".to_string())));
        }
        let mut ifr = ifreq::from_name(name)?;
        if let Err(err) = unsafe { get_iface_index(device.fd, &mut ifr) } {
            device.close();
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
            device.close();
            return Err(Box::new(err));
        }
        if let Err(err) = unsafe { get_iface_flags(device.fd, &mut ifr) } {
            device.close();
            return Err(Box::new(err));
        }
        unsafe {
            ifr.ifr_ifru.ifr_flags = ifr.ifr_ifru.ifr_flags | (libc::IFF_PROMISC as i16);
        }
        if let Err(err) = unsafe { get_iface_flags(device.fd, &mut ifr) } {
            device.close();
            return Err(Box::new(err));
        }
        Ok(Box::new(device))
    }
}

impl RawDevice for SocketDevice {
    fn type_(&self) -> Type {
        Type::Socket
    }
    fn name(&self) -> &String {
        &self.name
    }
    fn addr(&self) -> Result<[u8; ADDR_LEN], Box<dyn Error>> {
        let fd = socket(
            AddressFamily::Inet,
            SockType::Datagram,
            SockFlag::empty(),
            None,
        )?;
        let mut ifr = ifreq::from_name(self.name.as_str())?;
        unsafe {
            ifr.ifr_ifru.ifr_addr.sa_family = libc::AF_INET.try_into().unwrap();
        }
        if let Err(err) = unsafe { get_hwaddr(fd, &mut ifr) } {
            unsafe {
                unistd::close(fd)?;
            }
            Err(Box::new(err))
        } else {
            let addr = unsafe { ifr.ifr_ifru.ifr_hwaddr.sa_data };
            let addr =
                unsafe { &*(addr.as_ptr() as *const [i8; ADDR_LEN] as *const [u8; ADDR_LEN]) };
            unsafe {
                libc::close(fd);
            }
            Ok(*addr)
        }
    }
    fn close(&mut self) -> Result<(), Box<dyn Error>> {
        if self.fd != -1 {
            unistd::close(self.fd)? // TODO
        }
        // free device
        Ok(())
    }

    fn rx(&mut self, callback: fn(&Vec<u8>, usize, &Vec<u8>), arg: &Vec<u8>, timeout: i32) {
        let mut pfd = pollfd {
            fd: self.fd,
            events: POLLIN,
            revents: 0,
        };
        match unsafe { libc::poll(&mut pfd, 1, timeout) } {
            0 => return,
            -1 => eprintln!("poll"), // catch EINTR case
            _ => (),
        }
        let mut buf = vec![];
        buf.resize(2048, 0);
        let len: usize = match unsafe {
            libc::read(self.fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len())
        } {
            0 => return,
            -1 => {
                eprintln!("read");
                return;
            }
            len => len,
        }
        .try_into()
        .unwrap();
        callback(&buf, len, arg);
    }

    fn tx(&mut self, buf: &Vec<u8>) -> isize {
        unimplemented!()
    }
}
