use std::io::Result;

use bytes::{BytesMut, BufMut};
use byteorder::{BigEndian, ByteOrder};

use crate::protocol::common::addr::{IPV4_SIZE, IPV6_SIZE, ATYPE_IPV4, ATYPE_IPV6, ipv4_to_string, ipv6_to_string};
use std::convert::TryInto;

pub const VERSION: u8 = 1;

pub struct Request {
    version: u8,
    uuid: [u8; 16],
    command: u8,
    port: [u8; 2],
    atype: u8,
    addr: [u8; 16],
}

impl Request {
    pub fn new(version: u8, uuid: [u8; 16], command: u8, port: [u8; 2], atype: u8,
               addr: [u8; 16]) -> Request {
        return Request {
            version,
            uuid,
            command,
            port,
            atype,
            addr,
        };
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(64);
        buf.put_u8(self.version);
        buf.put_slice(&self.uuid);
        buf.put_u8(self.command);
        buf.put_slice(&self.port);
        buf.put_u8(self.atype);
        match self.atype {
            ATYPE_IPV4 => buf.put_slice(&self.addr[0..IPV4_SIZE]),
            ATYPE_IPV6 => buf.put_slice(&self.addr[0..IPV6_SIZE]),
            _ => buf.put_slice(&self.addr)
        }
        return buf.to_vec();
    }

    pub fn request_addr_port(&self) -> String {
        let addr = match self.atype {
            ATYPE_IPV4 => ipv4_to_string(self.addr[0..IPV4_SIZE].try_into().unwrap()),
            ATYPE_IPV6 => ipv6_to_string(self.addr),
            _ => String::from("")
        };
        let port = BigEndian::read_u16(&self.port);
        return format!("{}:{}", addr, port);
    }
}

pub struct Response {
    version: u8,
}

impl Response {
    pub fn new(version: u8) -> Response {
        return Response {
            version
        };
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        return [VERSION].to_vec();
    }
}
