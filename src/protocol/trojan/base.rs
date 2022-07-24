use crate::protocol::common::addr::IpAddress;
use crate::protocol::common::atype::Atype;
use crate::protocol::common::command::Command;
use crate::protocol::common::request::{InboundRequest, TransportProtocol};
use crate::proxy::base::SupportedProtocols;

use std::fmt;

pub const HEX_SIZE: usize = 56;
pub const CRLF: u16 = 0x0D0A;

pub struct Request {
    hex: Vec<u8>,
    command: Command,
    atype: Atype,
    addr: IpAddress,
    port: u16,
    proxy_protocol: SupportedProtocols,
}

impl Request {
    pub fn new(
        hex: Vec<u8>,
        command: Command,
        atype: Atype,
        addr: IpAddress,
        port: u16,
        proxy_protocol: SupportedProtocols,
    ) -> Request {
        return Request {
            hex,
            command,
            atype,
            addr,
            port,
            proxy_protocol,
        };
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
                self.proxy_protocol,
            ),
            _ => InboundRequest::new(
                self.atype,
                self.addr,
                self.command,
                self.port,
                TransportProtocol::TCP,
                self.proxy_protocol,
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
}

impl fmt::Display for Request {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "{} {}:{}",
            self.command.to_string(),
            self.addr.to_string(),
            self.port
        )
    }
}
