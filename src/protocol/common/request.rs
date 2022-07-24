use crate::protocol::common::atype::Atype;
use crate::protocol::common::command::Command;
use crate::{protocol::common::addr::IpAddress, proxy::base::SupportedProtocols};

use serde::{Deserialize, Serialize};
use log::error;
use std::net::{SocketAddr, ToSocketAddrs};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum TransportProtocol {
    TCP,
    UDP,
}

pub struct InboundRequest {
    pub atype: Atype,
    pub addr: IpAddress,
    pub command: Command,
    pub port: u16,
    pub transport_protocol: TransportProtocol,
    pub proxy_protocol: SupportedProtocols,
}

impl InboundRequest {
    #[inline]
    pub fn new(
        atype: Atype,
        addr: IpAddress,
        command: Command,
        port: u16,
        transport_protocol: TransportProtocol,
        proxy_protocol: SupportedProtocols,
    ) -> InboundRequest {
        InboundRequest {
            atype,
            addr,
            command,
            port,
            transport_protocol,
            proxy_protocol,
        }
    }

    #[inline]
    pub fn into_destination_address(&self) -> SocketAddr {
        let addrs: Vec<SocketAddr> = match (self.addr.to_string(), self.port)
            .to_socket_addrs() {
                Ok(sa) => sa.collect(),
                Err(_) => {
                    error!("Failed to lookup {}", self.addr.to_string());
                    panic!("");
                }
            };

        addrs.into_iter().nth(0).unwrap()
    }
}
