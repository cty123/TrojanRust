use log::debug;

use super::super::common::addr::{IPv4Addr, IPv6Addr};
use super::base::{AType, Command, Request};

macro_rules! march {
    ($ptr:ident, $i:expr) => {
        $ptr += $i;
    };
}

pub fn parse(buf: &[u8]) -> Result<Request, String> {
    let mut ptr = 0;

    let version = match buf[ptr] {
        5 => 0x5,
        _ => return Err(String::from("aaa")),
    };

    march!(ptr, 1);

    let command = match buf[ptr] {
        1 => Command::CONNECT,
        2 => Command::BIND,
        3 => Command::UDPASSOCIATE,
        _ => return Err(String::from("bbb")),
    };

    march!(ptr, 1);

    let rsv = buf[ptr];

    march!(ptr, 1);

    let atype = match buf[ptr] {
        1 => AType::IPv4,
        3 => AType::DOMAINNAME,
        4 => AType::IPv6,
        _ => return Err(String::from("ccc")),
    };

    march!(ptr, 1);

    let dest_addr = match atype {
        AType::IPv4 => IPv4Addr::new(buf, ptr),
        AType::DOMAINNAME => String::from(""),
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

    debug!("Received socks5 request: {}", request.dump_request());

    return Ok(request);
}
