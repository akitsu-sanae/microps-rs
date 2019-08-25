use super::TUNSETIFF;
use crate::util::RuntimeError;
use ifstructs::ifreq;
use libc::{self, ioctl, pollfd, IFF_NO_PI, IFF_TAP, POLLIN};
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
    let mut ifr = ifreq::from_name(name)?;
    ifr.set_flags(IFF_TAP as i16 | IFF_NO_PI as i16);
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
    let mut pfd = pollfd {
        fd: device.fd,
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
        match unsafe { libc::read(device.fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) } {
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

pub fn tx(device: &TapDevice, buf: &Vec<u8>, len: usize) {
    unsafe {
        libc::write(device.fd, buf.as_ptr() as *const libc::c_void, len);
    }
}

pub fn addr(name: &str, dst: &mut Vec<i8>, _size: usize) -> Result<(), Box<dyn Error>> {
    let socket = match unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) } {
        -1 => return Err(Box::new(RuntimeError::new("socket".to_string()))),

        socket => socket,
    };

    let mut ifr = ifreq::from_name(name)?;
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
    for (i, data) in unsafe { ifr.ifr_ifru.ifr_hwaddr.sa_data.iter().enumerate() } {
        dst[i] = *data;
    }
    unsafe {
        libc::close(socket);
    }
    Ok(())
}
