use super::{RawDevice, Type};
use crate::util::RuntimeError;
use ifstructs::ifreq;
use libc::{self, pollfd, IFF_NO_PI, IFF_TAP, POLLIN};
use nix::{fcntl, sys::stat::Mode, unistd, ioctl_write_int};
use std::error::Error;
use std::os::unix::io::{AsRawFd, RawFd};
use crate::ethernet::ADDR_LEN;

ioctl_write_ptr!(tun_set_iff, 'T', 202, libc::c_int);

pub struct TapDevice {
    fd: RawFd,
    name: String,
}

impl TapDevice {
    pub fn open(name: &str) -> Result<Box<dyn RawDevice>, Box<dyn Error>> {
        let mut device = TapDevice {
            fd: fcntl::open("/dev/net/tun", fcntl::OFlag::O_RDWR, Mode::empty())
                .expect("can not open /dev/net/tun"),
            name: name.to_string(),
        };
        if device.fd == -1 {
            device.close().unwrap();
            return Err(Box::new(RuntimeError::new(format!(
                            "can not open : {}",
                            name
            ))));
        }
        let mut ifr = ifreq::from_name(name)?;
        ifr.set_flags(IFF_TAP as i16 | IFF_NO_PI as i16);

        unsafe { tun_set_iff(device.fd, &mut ifr as *mut _ as *mut _) }?;
            // device.close().unwrap();
            //
        Ok(Box::new(device))
    }
}

impl RawDevice for TapDevice {
    fn type_(&self) -> Type {
        Type::Tap
    }
    fn name(&self) -> &String {
        &self.name
    }
    fn addr(&self) -> Result<[u8; ADDR_LEN], Box<dyn Error>> {
        let socket = match unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) } {
            -1 => return Err(Box::new(RuntimeError::new("socket".to_string()))),

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
            return Err(Box::new(RuntimeError::new(
                        "ioctl [SIOCGIFHWADDR]".to_string(),
            )));
        }
        let addr = unsafe { ifr.ifr_ifru.ifr_hwaddr.sa_data };
        let addr = unsafe { &*(addr.as_ptr() as *const [i8; ADDR_LEN] as *const [u8; ADDR_LEN]) };
        unsafe {
            libc::close(socket);
        }
        Ok(addr.clone())
    }

    fn close(&mut self) -> Result<(), Box<dyn Error>> {
        if self.fd != -1 {
            unistd::close(self.fd).expect("can not close fd")
        }
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
            -1 => eprintln!("poll"),
            _ => (),
        }
        let mut buf = vec![];
        buf.resize(2048, 0);
        use std::convert::TryInto;
        let len: usize =
            match unsafe { libc::read(self.fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) } {
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
        unsafe {
            libc::write(self.fd, buf.as_ptr() as *const libc::c_void, buf.len())
        }
    }
}

