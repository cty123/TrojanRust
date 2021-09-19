use crate::protocol::common::addr::IpAddress;
use crate::protocol::common::atype::Atype;
use crate::protocol::common::command::Command;
use crate::protocol::common::request::{InboundRequest, TransportProtocol};

pub const HEX_SIZE: usize = 56;
pub const CRLF: u16 = 0x0D0A;

pub struct Request {
    hex: [u8; HEX_SIZE],
    command: Command,
    atype: Atype,
    addr: IpAddress,
    port: u16,
}

impl Request {
    pub fn new(
        hex: [u8; HEX_SIZE],
        command: Command,
        atype: Atype,
        addr: IpAddress,
        port: u16,
    ) -> Request {
        return Request {
            hex,
            command,
            atype,
            addr,
            port,
        };
    }

    #[inline]
    pub fn request_addr_port(&self) -> String {
        return format!("{}:{}", self.addr.to_string(), self.port);
    }

    #[inline]
    pub fn dump_request(&self) -> String {
        return format!(
            "[{} => {}]",
            self.command.to_string(),
            self.request_addr_port()
        );
    }

    #[inline]
    pub fn inbound_request(self) -> InboundRequest {
        return match self.command {
            Command::Udp => InboundRequest::new(
                self.atype,
                self.addr,
                self.command,
                self.port,
                TransportProtocol::UDP,
            ),
            _ => InboundRequest::new(
                self.atype,
                self.addr,
                self.command,
                self.port,
                TransportProtocol::TCP,
            ),
        };
    }

    #[inline]
    pub fn validate(&self, secret: &[u8]) -> bool {
        if secret.len() != self.hex.len() {
            return false;
        }

        return secret == self.hex;
    }

    #[inline]
    pub fn from_request(request: &InboundRequest, secret: [u8; 56]) -> Request {
        Request {
            hex: secret,
            command: request.command,
            atype: request.atype,
            addr: request.addr.clone(),
            port: request.port,
        }
    }
}
