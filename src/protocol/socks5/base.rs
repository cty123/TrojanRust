use std::net::IpAddr;
use bytes::{BytesMut, BufMut};

use crate::protocol::common::addr::IpAddress;

pub struct Request {
    version: u8,
    command: u8,
    rsv: u8,
    atype: u8,
    addr: IpAddress,
    port: u16,
}

pub struct ClientHello {
    version: u8,
    method_size: u8,
    // Assume for now that the number of methods is always 1
    methods: u8,
}

pub struct ServerHello {
    version: u8,
    method: u8,
}

pub struct RequestAck {
    version: u8,
    rep: u8,
    rsv: u8,
    atype: u8,
    addr: IpAddress,
    port: u16,
}

impl ServerHello {
    pub fn new(version: u8, method: u8) -> ServerHello {
        return ServerHello {
            version,
            method,
        };
    }

    pub fn to_bytes(&self) -> [u8; 2] {
        return [self.version, self.method];
    }
}

impl RequestAck {
    pub fn new(version: u8, rep: u8, rsv: u8, atype: u8, addr: IpAddress, port: u16) -> RequestAck {
        return RequestAck {
            version,
            rep,
            rsv,
            atype,
            addr,
            port,
        };
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(128);
        buf.put_slice(&[self.version, self.rep, self.rsv, 1]);
        buf.put_slice(&self.addr.to_bytes());
        buf.put_u16(self.port);
        return buf.to_vec();
    }
}

impl Request {
    pub fn new(
        version: u8,
        command: u8,
        rsv: u8,
        atype: u8,
        port: u16,
        addr: IpAddress,
    ) -> Request {
        return Request {
            version,
            command,
            rsv,
            atype,
            port,
            addr,
        };
    }

    pub fn request_addr_port(&self) -> String {
        return format!("{}:{}", self.addr.to_string(), self.port);
    }

    pub fn dump_request(&self) -> String {
        return format!(
            "[{} => {}:{}]",
            self.get_command(),
            self.addr.to_string(),
            self.port
        );
    }

    fn get_command(&self) -> &str {
        return match self.command {
            1 => "Connect",
            2 => "Bind",
            3 => "UDP Associate",
            _ => "Unsupported"
        }
    }
}
