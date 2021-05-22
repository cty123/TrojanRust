use bytes::{BytesMut, BufMut};
use byteorder::{BigEndian, ByteOrder};
use std::io::Result;

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

    pub fn dump_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(64);
        buf.put_u8(self.version);
        buf.put_slice(&self.uuid);
        buf.put_u8(self.command);
        buf.put_u8(self.atype);
        buf.put_slice(&self.addr);
        buf.put_slice(&self.port);
        return buf.to_vec();
    }

    pub fn request_addr_port(&self) -> String {
        let addr = self.addr.iter().map(|i| i.to_string()).collect::<String>();
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
}
