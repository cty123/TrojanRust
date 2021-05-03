use std::fmt::{Display, Formatter, Result};

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

pub struct Ack {
    version: u8,
    method: u8,
}

pub struct RequestAck {
    version: u8,
    rep: u8,
    rsv: u8,
    atype: AType,
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
            version: version,
            command: command,
            rsv: rsv,
            atype: atype,
            dest_addr: dest_addr,
            dest_port: dest_port,
        };
    }

    pub fn request_addr_port(&self) -> String {
        return format!("{}:{}", self.dest_addr, self.dest_port);
    }

    pub fn dump_request(&self) -> String {
        return format!(
            "command: {} {}::{}:{}",
            self.command.to_string(),
            self.atype.to_string(),
            self.dest_addr,
            self.dest_port
        );
    }
}
