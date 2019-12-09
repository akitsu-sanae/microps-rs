use super::{RawDevice, Type};
use crate::buffer::Buffer;
use crate::ethernet::{MacAddr, ADDR_LEN};
use crate::util::RuntimeError;
use ifstructs::ifreq;
use libc::{self, pollfd, IFF_NO_PI, IFF_TAP, POLLIN};
use nix::{
    errno::{errno, Errno},
    fcntl,
    sys::stat::Mode,
    unistd,
};
use std::error::Error;
use std::os::unix::io::RawFd;
use std::sync::Arc;
use std::thread::JoinHandle;

ioctl_write_ptr!(tun_set_iff, 'T', 202, libc::c_int);

#[derive(Debug, Clone)]
pub struct Device {
    fd: RawFd,
    name: String,
}

impl Device {
    pub fn open(name: &str) -> Result<Arc<dyn RawDevice + Sync + Send>, Box<dyn Error>> {
        let device = Device {
            fd: fcntl::open("/dev/net/tun", fcntl::OFlag::O_RDWR, Mode::empty())
                .expect("can not open /dev/net/tun"),
            name: name.to_string(),
        };
        if device.fd == -1 {
            device.close().unwrap();
            return Err(RuntimeError::new(format!("can not open : {}", name)));
        }
        let mut ifr = ifreq::from_name(name)?;
        ifr.set_flags(IFF_TAP as i16 | IFF_NO_PI as i16);

        unsafe { tun_set_iff(device.fd, &mut ifr as *mut _ as *mut _) }?;
        // device.close().unwrap();
        //
        Ok(Arc::new(device))
    }
}

impl RawDevice for Device {
    fn type_(&self) -> Type {
        Type::Tap
    }
    fn name(&self) -> &String {
        &self.name
    }
    fn addr(&self) -> Result<MacAddr, Box<dyn Error>> {
        let socket = match unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) } {
            -1 => return Err(RuntimeError::new("socket".to_string())),

            socket => socket,
        };

        let mut ifr = ifreq::from_name(self.name.as_str())?;
        ifr.ifr_ifru.ifr_addr = libc::sockaddr {
            sa_family: libc::AF_INET as u16,
            sa_data: [0; 14],
        };

        if unsafe { libc::ioctl(socket, libc::SIOCGIFHWADDR, &ifr) } == -1 {
            unsafe {
                libc::close(socket);
            }
            return Err(RuntimeError::new("ioctl [SIOCGIFHWADDR]".to_string()));
        }
        let addr = unsafe { ifr.ifr_ifru.ifr_hwaddr.sa_data };
        let addr = unsafe { &*(addr.as_ptr() as *const [i8; ADDR_LEN] as *const [u8; ADDR_LEN]) };
        unsafe {
            libc::close(socket);
        }
        Ok(MacAddr(*addr))
    }

    fn close(&self) -> Result<(), Box<dyn Error>> {
        if self.fd != -1 {
            unistd::close(self.fd).expect("can not close fd")
        }
        Ok(())
    }
    fn rx(
        &self,
        callback: Box<dyn FnOnce(Buffer) -> Result<Option<JoinHandle<()>>, Box<dyn Error>>>,
        timeout: i32,
    ) -> Result<Option<JoinHandle<()>>, Box<dyn Error>> {
        let mut pfd = pollfd {
            fd: self.fd,
            events: POLLIN,
            revents: 0,
        };
        match unsafe { libc::poll(&mut pfd, 1, timeout) } {
            0 => return Ok(None), // timeout
            -1 => {
                if errno() != Errno::EINTR as i32 {
                    return Err(RuntimeError::new("poll error".to_string()));
                } else {
                    return Ok(None);
                }
            }
            _ => (),
        }
        let mut buf = vec![];
        buf.resize(2048, 0);
        use std::convert::TryInto;
        let len: usize = match unsafe {
            libc::read(self.fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len())
        } {
            0 => return Ok(None),
            -1 => return Err(RuntimeError::new("read error".to_string())),
            len => len,
        }
        .try_into()
        .unwrap();
        buf.resize(len, 0);
        callback(Buffer::from_vec(buf))
    }
    fn tx(&self, buf: Buffer) -> Result<(), Box<dyn Error>> {
        let buf = buf.to_vec();
        unsafe { libc::write(self.fd, buf.as_ptr() as *const libc::c_void, buf.len()) };
        Ok(())
    }
}
