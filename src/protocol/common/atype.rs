use std::fmt::{self};
use std::io::{Error, ErrorKind, Result};

const ATYPE_IPV4: u8 = 1;
const ATYPE_DOMAIN_NAME: u8 = 3;
const ATYPE_IPV6: u8 = 4;

#[derive(Copy, Clone)]
pub enum Atype {
    IPv4,
    IPv6,
    DomainName,
}

impl Atype {
    #[inline]
    pub fn from(atype: u8) -> Result<Atype> {
        match atype {
            ATYPE_IPV4 => Ok(Atype::IPv4),
            ATYPE_IPV6 => Ok(Atype::IPv6),
            ATYPE_DOMAIN_NAME => Ok(Atype::DomainName),
            _ => Err(Error::new(
                ErrorKind::Unsupported,
                "Unsupported address type",
            )),
        }
    }

    #[inline]
    pub fn to_byte(&self) -> u8 {
        match self {
            Atype::IPv4 => ATYPE_IPV4,
            Atype::IPv6 => ATYPE_IPV6,
            Atype::DomainName => ATYPE_DOMAIN_NAME,
        }
    }
}

impl fmt::Display for Atype {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Atype::IPv4 => write!(fmt, "IPv4"),
            Atype::IPv6 => write!(fmt, "IPv6"),
            Atype::DomainName => write!(fmt, "DomainName"),
        }
    }
}
