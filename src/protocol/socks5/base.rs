use crate::protocol::common::addr::AType;
use crate::protocol::common::command::Command;
use crate::protocol::vless;

use std::convert::TryInto;
use futures::StreamExt;
use itertools::Itertools;

pub struct Request {
    version: u8,
    command: u8,
    rsv: u8,
    atype: u8,
    addr: [u8; 16],
    port: [u8; 2],
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
    // Assume for now that we only use IPv4 for server
    addr: [u8; 4],
    port: [u8; 2],
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
    pub fn new(version: u8, rep: u8, rsv: u8, atype: u8, addr: [u8; 4], port: [u8; 2]) -> RequestAck {
        return RequestAck {
            version,
            rep,
            rsv,
            atype,
            addr,
            port,
        };
    }

    pub fn to_bytes(&self) -> [u8; 10] {
        return [self.version, self.rep, self.rsv, 1,
            self.addr[0], self.addr[1], self.addr[2], self.addr[3],
            self.port[0], self.port[1]];
    }
}

impl Request {
    pub fn new(
        version: u8,
        command: u8,
        rsv: u8,
        atype: u8,
        port: [u8; 2],
        addr: [u8; 16],
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
        return format!("{}:{}", self.get_addr(), self.get_port());
    }

    pub fn dump_request(&self) -> String {
        return format!(
            "[{} {}::{}:{}]",
            self.get_command(),
            self.get_atype(),
            self.get_addr(),
            self.get_port()
        );
    }

    pub fn to_vless_request(&self) -> vless::base::Request {
        return vless::base::Request::new(0, [0; 16], 1, self.port, 1, self.addr);
    }

    fn get_addr(&self) -> String {
        return match self.atype {
            1 => self.addr[0..4].to_vec().iter().join(":"),
            4 => self.addr.to_vec().into_iter()
                .map(|i| i.to_string())
                .join(":"),
            _ => "Unsupported".parse().unwrap()
        };
    }

    fn get_port(&self) -> u16 {
        return ((self.port[0] as u16) << 8) | self.port[1] as u16;
    }

    fn get_command(&self) -> &str {
        return match self.command {
            1 => "Connect",
            2 => "Bind",
            3 => "UDP Associate",
            _ => "Unsupported"
        }
    }

    fn get_atype(&self) -> &str {
       return match self.atype {
           1 => "IPv4",
           3 => "DomainName",
           4 => "IPv6",
           _ => "Unsupported"
       }
    }
}
