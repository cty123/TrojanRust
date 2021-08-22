use bytes::{BufMut, BytesMut};

use crate::protocol::common::addr::IpAddress;
use crate::protocol::common::atype::Atype;
use crate::protocol::common::command::Command;
use crate::protocol::common::request::{InboundRequest, TransportProtocol};

const CRLF: u16 = 0x0D0A;

pub struct Request {
    hex: [u8; 56],
    command: Command,
    atype: Atype,
    addr: IpAddress,
    port: u16,
}

impl Request {
    pub fn new(
        hex: [u8; 56],
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

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(128);
        buf.put_slice(&self.hex);
        buf.put_u16(CRLF);
        buf.put_u8(self.command.to_byte());
        buf.put_u8(self.atype.to_byte());
        buf.put_slice(&self.addr.to_bytes_vec());
        buf.put_u16(self.port);
        buf.put_u16(CRLF);
        return buf.to_vec();
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
