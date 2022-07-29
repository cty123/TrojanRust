use crate::protocol::common::addr::IpAddrPort;
use crate::protocol::common::atype::Atype;
use crate::protocol::common::command::Command;
use crate::{protocol::common::addr::IpAddress, proxy::base::SupportedProtocols};

use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum TransportProtocol {
    TCP,
    UDP,
}

pub struct InboundRequest {
    pub atype: Atype,
    pub addr_port: IpAddrPort,
    pub command: Command,
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
    ) -> Self {
        Self {
            atype,
            addr_port: IpAddrPort::new(addr, port),
            command,
            transport_protocol,
            proxy_protocol,
        }
    }
}
