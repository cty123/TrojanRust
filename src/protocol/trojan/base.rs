use bytes::{BufMut, BytesMut};

use crate::protocol::common::addr::IpAddress;
use crate::protocol::common::command::{BIND, CONNECT, UDP};
use crate::protocol::common::request::{InboundRequest, TransportProtocol};

const CRLF: u16 = 0x0D0A;

pub struct Request {
    hex: [u8; 56],
    command: u8,
    atype: u8,
    addr: IpAddress,
    addr_len: usize,
    port: u16,
}

impl Request {
    pub fn new(
        hex: [u8; 56],
        command: u8,
        atype: u8,
        addr: IpAddress,
        addr_len: usize,
        port: u16,
    ) -> Request {
        return Request {
            hex,
            command,
            atype,
            addr,
            addr_len,
            port,
        };
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(128);
        buf.put_slice(&self.hex);
        buf.put_u16(CRLF);
        buf.put_u8(self.command);
        buf.put_u8(self.atype);
        buf.put_slice(&self.addr.to_bytes());
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
        let command = match self.command {
            CONNECT => "Connect",
            BIND => "Bind",
            UDP => "UDP Associate",
            _ => "Unsupported",
        };
        return format!("[{} => {}]", command, self.request_addr_port());
    }

    #[inline]
    pub fn inbound_request(self) -> InboundRequest {
        return match self.command {
            UDP => InboundRequest::new(
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

    pub fn from_request(request: &InboundRequest, secret: [u8; 56]) -> Request {
        Request {
            hex: secret,
            command: request.command,
            atype: request.atype,
            addr: request.addr.clone(),
            addr_len: request.addr.len(),
            port: request.port,
        }
    }
}
