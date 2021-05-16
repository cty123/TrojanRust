use log::debug;

use super::super::common::addr::{IPv4Addr, IPv6Addr};
use super::base::{AType, Command, Request};
use crate::protocol::common::addr::DomainName;
use std::io::{Error, ErrorKind};
use std::io::Result;

macro_rules! march {
    ($ptr:ident, $i:expr) => {
        $ptr += $i;
    };
}

pub fn parse(buf: &[u8]) -> Result<Request> {
    let mut ptr = 0;

    let version = match buf[ptr] {
        5 => 0x5,
        _ => return Err(Error::new(ErrorKind::InvalidInput, "Failed to parse request data")),
    };

    march!(ptr, 1);

    let command = match buf[ptr] {
        1 => Command::CONNECT,
        2 => Command::BIND,
        3 => Command::UDPASSOCIATE,
        _ => return Err(Error::new(ErrorKind::InvalidInput, "Failed to parse request data")),
    };

    march!(ptr, 1);

    let rsv = buf[ptr];

    march!(ptr, 1);

    let atype = match buf[ptr] {
        1 => AType::IPv4,
        3 => AType::DOMAINNAME,
        4 => AType::IPv6,
        _ => return Err(Error::new(ErrorKind::InvalidInput, "Failed to parse request data")),
    };

    march!(ptr, 1);

    let dest_addr = match atype {
        AType::IPv4 => IPv4Addr::new(buf, ptr),
        AType::DOMAINNAME => DomainName::new(buf, ptr),
        AType::IPv6 => IPv6Addr::new(buf, ptr),
    };

    match atype {
        AType::IPv4 => march!(ptr, 4),
        AType::DOMAINNAME => march!(ptr, 256),
        AType::IPv6 => march!(ptr, 16),
    }

    let dest_port = u16::from_be_bytes([buf[ptr], buf[ptr + 1]]);

    march!(ptr, 2);

    let request = Request::new(version, command, rsv, atype, dest_addr, dest_port);

    return Ok(request);
}
