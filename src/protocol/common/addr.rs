use std::fmt::{self};
use std::net::IpAddr;

pub const IPV4_SIZE: usize = 4;
pub const IPV6_SIZE: usize = 16;
pub const DOMAIN_NAME_SIZE: usize = 256;

pub const ATYPE_IPV4: u8 = 1;
pub const ATYPE_IPV6: u8 = 4;
pub const ATYPE_DOMAIN_NAME: u8 = 3;

pub enum IpAddress {
    IpAddr(IpAddr),
    Domain(DomainName),
}

pub struct DomainName {
    inner: [u8; 256],
    size: usize,
}

impl fmt::Display for DomainName {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "Domain:{}", String::from_utf8_lossy(&self.inner))
    }
}

impl fmt::Display for IpAddress {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IpAddress::IpAddr(IpAddr::V4(ip)) => ip.fmt(fmt),
            IpAddress::IpAddr(IpAddr::V6(ip)) => ip.fmt(fmt),
            IpAddress::Domain(domain) => domain.fmt(fmt),
        }
    }
}

impl IpAddress {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            IpAddress::IpAddr(IpAddr::V4(ip)) => ip.octets().to_vec(),
            IpAddress::IpAddr(IpAddr::V6(ip)) => ip.octets().to_vec(),
            IpAddress::Domain(domain) => domain.inner[..domain.size].to_vec(),
        }
    }
}
