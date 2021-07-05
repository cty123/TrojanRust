use std::net::IpAddr;
use std::fmt::{self};

pub const IPV4_SIZE: usize = 4;
pub const IPV6_SIZE: usize = 16;
pub const DOMAIN_NAME_SIZE: usize = 256;

pub const ATYPE_IPV4: u8 = 1;
pub const ATYPE_IPV6: u8 = 4;
pub const ATYPE_DOMAIN_NAME: u8 = 3;

pub enum IpAddress {
    IpAddr(IpAddr),
    Domain(DomainName)
}

pub struct DomainName {
    inner: [u8; 256]
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
            IpAddress::Domain(domain) => domain.fmt(fmt)
        }
    }
}

impl IpAddress {
    pub fn to_bytes() {
        unimplemented!()
    }
}
