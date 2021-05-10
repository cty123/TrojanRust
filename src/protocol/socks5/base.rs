use std::fmt::{Display, Formatter, Result};
use crate::protocol::common::packet::Packet;
use std::convert::TryInto;

pub enum Command {
    CONNECT = 1,
    BIND = 2,
    UDPASSOCIATE = 3,
}

pub enum AType {
    IPv4 = 1,
    DOMAINNAME = 3,
    IPv6 = 4,
}

pub struct Request {
    version: u8,
    command: Command,
    rsv: u8,
    atype: AType,
    dest_addr: String,
    dest_port: u16,
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
    bind_addr: [u8; 4],
    bind_port: [u8; 2],
}

impl Display for Command {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Command::CONNECT => write!(f, "CONNECT"),
            Command::BIND => write!(f, "BIND"),
            Command::UDPASSOCIATE => write!(f, "UDPASSOCIATE"),
        }
    }
}

impl Display for AType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            AType::IPv4 => write!(f, "ipv4"),
            AType::DOMAINNAME => write!(f, "domain"),
            AType::IPv6 => write!(f, "ipv6"),
        }
    }
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
    pub fn new(version: u8, rep: u8, rsv: u8, atype: u8, bind_addr: [u8; 4], bind_port: [u8; 2]) -> RequestAck {
        return RequestAck {
            version,
            rep,
            rsv,
            atype,
            bind_addr,
            bind_port,
        };
    }

    pub fn to_bytes(&self) -> [u8; 10] {
        return [self.version, self.rep, self.rsv, 1,
            self.bind_addr[0], self.bind_addr[1], self.bind_addr[2], self.bind_addr[3],
            self.bind_port[0], self.bind_port[1]];
    }
}

impl Request {
    pub fn new(
        version: u8,
        command: Command,
        rsv: u8,
        atype: AType,
        dest_addr: String,
        dest_port: u16,
    ) -> Request {
        return Request {
            version,
            command,
            rsv,
            atype,
            dest_addr,
            dest_port,
        };
    }

    pub fn request_addr_port(&self) -> String {
        return format!("{}:{}", self.dest_addr, self.dest_port);
    }

    pub fn dump_request(&self) -> String {
        return format!(
            "[{} {}::{}:{}]",
            self.command.to_string(),
            self.atype.to_string(),
            self.dest_addr,
            self.dest_port
        );
    }
}
