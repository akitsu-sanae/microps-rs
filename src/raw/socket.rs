use std::error::Error;
use super::RawDevice;

pub struct SocketDevice {
    pub fd: i32,
}

impl SocketDevice {
    pub fn open(name: &str) -> Result<Box<dyn RawDevice>, Box<dyn Error>> {
        /*
           let dev = SocketDevice {
           fd: socket(PF_PACKET, SOCK_RAW, htons(ETH_P_ALL)),
           };
           if dev.fd == -1 {
           close(dev);
           return Err("socket failed".to_string());
           }
           let ifr = ifreq::from_name(name)?;
           if ioctl(dev.fs, SIOCGIFINDEX, &ifr) == -1 {
           close(dev);
           return Err("ioctl [SIOCGIFINDEX]".to_string());
           }
           memset(&sockaddr, 0x00, sizeof(sockaddr));
           sockaddr.sll_family = AF_PACKET;
           sockaddr.sll_protocal = htons(ETH_P_ALL);
           sockaddr.sll_ifindex = ifr.ifr_ifindex;
           if bind(dev.fd, &socketaddr, sizeof(sockaddr)) == -1 {
           return Err("bind".to_string());
           }
           if ioctl(dev.fd, SIOCGIFFLAGS, &ifr) == -1 {
           return Err("".to_string());
           } */
        unimplemented!()
    }
}


