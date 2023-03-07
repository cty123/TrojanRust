use bytes::Bytes;
use std::fmt::{self};
use std::io::{Error, ErrorKind};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs};

pub const IPV4_SIZE: usize = 4;
pub const IPV6_SIZE: usize = 16;

/// DomainName is a vector of bytes whose length can go up to 255. This is not the most efficient way of
/// storing DomainName, should consider using stack memory to avoid calling malloc and free repeatedly.
/// This may be altered after we perform a thourough benchmark to determine the tradeoffs between slice and Vec.
pub struct DomainName {
    inner: Bytes,
}

/// Wrap around std::net::IpAddr to create a parent enum for all kinds of IpAddresses used in Trojan.
/// Apart from the basic IPv4 and IPv6 types provided by standard library, DomainName is commonly used
/// for proxy protocols, so we need to extend that.
pub enum IpAddress {
    IpAddr(IpAddr),
    Domain(DomainName),
}

/// Wrapper class that contains the destination ip and port of the proxy request.
/// The struct is capable of converting to SocketAddr class that can be used to establish an outbound connection.
pub struct IpAddrPort {
    pub ip: IpAddress,
    pub port: u16,
}

/// Expose constructor for IpAddrPort for the easy of initialization
impl IpAddrPort {
    #[inline]
    pub fn new(ip: IpAddress, port: u16) -> Self {
        Self { ip, port }
    }
}

/// IpAddrPort is essentially SocketAddr, except we allow DomainName which needs to be resolved by DNS query.
/// TODO: Come up with a better way of resolving DNS names by adding builtin DNS cache in memory or make pluggable modules.
impl Into<std::io::Result<SocketAddr>> for IpAddrPort {
    fn into(self) -> std::io::Result<SocketAddr> {
        match self.ip {
            IpAddress::IpAddr(addr) => Ok(SocketAddr::new(addr, self.port)),
            IpAddress::Domain(domain) => {
                let name = match std::str::from_utf8(&domain.inner) {
                    Ok(name) => name,
                    Err(_) => {
                        return Err(Error::new(
                            ErrorKind::InvalidData,
                            "request domain name contains non-utf8 character",
                        ))
                    }
                };

                // to_socket_addrs function implicitly runs a DNS query to resolve the DomainName
                let addrs = match (name, self.port).to_socket_addrs() {
                    Ok(a) => a,
                    Err(_) => {
                        return Err(Error::new(
                            ErrorKind::AddrNotAvailable,
                            "Failed to resolve DNS name",
                        ));
                    }
                };

                return match addrs.into_iter().nth(0) {
                    Some(n) => Ok(n),
                    None => Err(Error::new(
                        ErrorKind::AddrNotAvailable,
                        "Failed to resolve DNS name",
                    )),
                };
            }
        }
    }
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
    pub fn from_bytes(addr: Bytes) -> IpAddress {
        IpAddress::Domain(DomainName { inner: addr })
    }
}

impl DomainName {
    pub fn as_bytes(&self) -> &[u8] {
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
