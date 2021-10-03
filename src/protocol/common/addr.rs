use std::fmt::{self};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

pub const IPV4_SIZE: usize = 4;
pub const IPV6_SIZE: usize = 16;

/// Wrap around std::net::IpAddr to create a parent enum for all kinds of IpAddresses used in Trojan.
/// Apart from the basic IPv4 and IPv6 types provided by standard library, DomainName is commonly used
/// for proxy protocols, so we need to extend that.
#[derive(Clone)]
pub enum IpAddress {
    IpAddr(IpAddr),
    Domain(DomainName),
}

/// DomainName is a vector of bytes whose length can go up to 256. This is not the most efficient way of
/// storing DomainName, should consider using stack memory to avoid calling malloc and free repeatedly.
/// This may be altered after we perform a thourough benchmark to determine the tradeoffs between slice and Vec.
#[derive(Clone)]
pub struct DomainName {
    inner: Vec<u8>,
}

impl IpAddress {
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            IpAddress::IpAddr(IpAddr::V4(_)) => IPV4_SIZE,
            IpAddress::IpAddr(IpAddr::V6(_)) => IPV6_SIZE,
            IpAddress::Domain(domain) => domain.inner.len(),
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

impl DomainName {
    pub fn to_bytes(&self) -> &[u8] {
        &self.inner
    }
}

impl fmt::Display for DomainName {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}", String::from_utf8_lossy(&self.inner))
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
