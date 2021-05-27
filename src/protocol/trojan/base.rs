use std::convert::TryInto;

use bytes::{BytesMut, BufMut};
use byteorder::{BigEndian, ByteOrder};

use crate::protocol::common::addr::{IPV4_SIZE, IPV6_SIZE, DOMAIN_NAME_SIZE, ATYPE_IPV4, ATYPE_IPV6,
                                    ATYPE_DOMAIN_NAME, ipv4_to_string, ipv6_to_string};
use crate::protocol::common::command::{CONNECT, UDP_ASSOCIATE};

pub struct Request {
    hex: [u8; 56],
    command: u8,
    atype: u8,
    addr: [u8; 256],
    addr_len: usize,
    port: [u8; 2],
}

impl Request {
    pub fn new(hex: [u8; 56], command: u8, atype: u8, addr: [u8; 256], addr_len: usize, port: [u8; 2]) -> Request {
        return Request {
            hex,
            command,
            atype,
            addr,
            addr_len,
            port,
        };
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(128);
        buf.put_slice(&self.hex);
        buf.put_u8(0x0D);
        buf.put_u8(0x0A);
        buf.put_u8(self.command);
        buf.put_u8(self.atype);
        match self.atype {
            ATYPE_IPV4 => buf.put_slice(&self.addr[0..IPV4_SIZE]),
            ATYPE_IPV6 => buf.put_slice(&self.addr[0..IPV6_SIZE]),
            _ => buf.put_slice(&self.addr)
        }
        buf.put_slice(&self.port);
        buf.put_u8(0x0D);
        buf.put_u8(0x0A);
        return buf.to_vec();
    }

    pub fn request_addr_port(&self) -> String {
        let addr = match self.atype {
            ATYPE_IPV4 => ipv4_to_string(self.addr[0..IPV4_SIZE].try_into().unwrap()),
            ATYPE_IPV6 => ipv6_to_string(self.addr[0..IPV6_SIZE].try_into().unwrap()),
            ATYPE_DOMAIN_NAME => String::from_utf8_lossy(&self.addr[0..self.addr_len]).to_string(),
            _ => String::from("Unknown")
        };
        let port = BigEndian::read_u16(&self.port);
        return format!("{}:{}", addr, port);
    }

    pub fn dump_request(&self) -> String {
        let command = match self.command {
            CONNECT => "Connect",
            UDP_ASSOCIATE => "UDP Associate",
            _ => "Unsupported"
        };

        let atype = match self.atype {
            ATYPE_IPV4 => "IPv4",
            ATYPE_IPV6 => "IPv6",
            ATYPE_DOMAIN_NAME => "DomainName",
            _ => "Unsupported"
        };

        return format!(
            "[{} {}::{}]",
            command,
            atype,
            self.request_addr_port(),
        );
    }
}
