use crate::protocol::common::addr::IpAddress;
use crate::protocol::common::atype::Atype;
use crate::protocol::common::command::Command;
use crate::protocol::common::request::{InboundRequest, TransportProtocol};
use crate::proxy::base::SupportedProtocols;

use std::fmt;

/// Trojan hex payload is always 56 bytes
pub const HEX_SIZE: usize = 56;

/// Trojan protocol uses the 0x0D0A as deliminate for packet header and payload
pub const CRLF: u16 = 0x0D0A;

pub struct Request {
    command: Command,
    atype: Atype,
    addr: IpAddress,
    port: u16,
    proxy_protocol: SupportedProtocols,
}

impl Request {
    pub fn new(
        command: Command,
        atype: Atype,
        addr: IpAddress,
        port: u16,
        proxy_protocol: SupportedProtocols,
    ) -> Request {
        return Request {
            command,
            atype,
            addr,
            port,
            proxy_protocol,
        };
    }

    #[inline]
    pub fn into_request(self) -> InboundRequest {
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
