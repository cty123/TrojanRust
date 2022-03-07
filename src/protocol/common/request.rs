use crate::protocol::common::addr::IpAddress;
use crate::protocol::common::atype::Atype;
use crate::protocol::common::command::Command;

use serde::{Deserialize, Serialize};
use std::net::{SocketAddr, ToSocketAddrs};

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
        // let addrs: Vec<SocketAddr> = format!("{}:{}", self.addr.to_string(), self.port)
        //     .to_socket_addrs()
        //     .unwrap()
        //     .collect()

        // addrs.first().unwrap()

        let addrs: Vec<SocketAddr> = (self.addr.to_string(), self.port)
            .to_socket_addrs()
            .unwrap()
            .collect();

        addrs.into_iter().nth(0).unwrap()
    }
}
