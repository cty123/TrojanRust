use std::fmt::{self};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

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
    inner: Vec<u8>,
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
            IpAddress::Domain(domain) => domain.inner.to_vec(),
        }
    }

    #[inline]
    pub fn from_u32(addr: u32) -> IpAddress {
        IpAddress::IpAddr(IpAddr::V4(Ipv4Addr::from(addr)))
    }

    #[inline]
    pub fn from_u128(addr: u128) -> IpAddress {
        IpAddress::IpAddr(IpAddr::V6(Ipv6Addr::from(addr)))
    }

    #[inline]
    pub fn from_vec(addr: Vec<u8>) -> IpAddress {
        IpAddress::Domain(DomainName { inner: addr })
    }
}
