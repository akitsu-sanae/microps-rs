use super::{ifreq, TUNSETIFF};
use crate::util::RuntimeError;
use libc::{ioctl, IFF_NO_PI, IFF_TAP};
use nix::{dir::Dir, fcntl::OFlag, sys::stat::Mode, unistd};
use std::error::Error;
use std::os::unix::io::{AsRawFd, RawFd};

pub struct TapDevice {
    fd: RawFd,
}

pub fn open(name: &str) -> Result<TapDevice, Box<dyn Error>> {
    let device = TapDevice {
        fd: Dir::open("/dev/net/tun", OFlag::O_RDWR, Mode::empty())
            .unwrap()
            .as_raw_fd(),
    };
    if device.fd == -1 {
        close(&device);
        return Err(Box::new(RuntimeError::new(format!(
            "can not open : {}",
            name
        ))));
    }
    let mut ifr = ifreq::default();
    ifr.ifr_flags = IFF_TAP | IFF_NO_PI;
    if unsafe { ioctl(device.fd, TUNSETIFF, &ifr) } == -1 {
        close(&device);
        return Err(Box::new(RuntimeError::new("octl [TUNSETIFF]".to_string())));
    }
    Ok(device)
}

pub fn close(device: &TapDevice) {
    if device.fd != -1 {
        unistd::close(device.fd).unwrap()
    }
    // free(device)
}

pub fn rx(
    device: &TapDevice,
    callback: fn(&Vec<u8>, usize, &Vec<u8>),
    arg: &Vec<u8>,
    timeout: i32,
) {
    unimplemented!()
}

pub fn tx(device: &TapDevice, buf: &Vec<u8>, len: usize) {
    unimplemented!()
}

pub fn addr(name: &str, dst: &Vec<u8>, size: usize) {
    unimplemented!()
}
