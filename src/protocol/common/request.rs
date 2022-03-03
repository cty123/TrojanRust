use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::protocol::common::addr::IpAddress;
use crate::protocol::common::atype::Atype;
use crate::protocol::common::command::Command;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum TransportProtocol {
    TCP,
    UDP,
    GRPC,
}

pub struct InboundRequest {
    pub atype: Atype,
    pub addr: IpAddress,
    pub command: Command,
    pub port: u16,
    pub transport_protocol: TransportProtocol,
}

impl InboundRequest {
    #[inline]
    pub fn new(
        atype: Atype,
        addr: IpAddress,
        command: Command,
        port: u16,
        transport_protocol: TransportProtocol,
    ) -> InboundRequest {
        InboundRequest {
            atype,
            addr,
            command,
            port,
            transport_protocol,
        }
    }

    #[inline]
    pub fn into_destination_address(&self) -> SocketAddr {
        format!("{}:{}", self.addr, self.port).parse().unwrap()
    }
}
