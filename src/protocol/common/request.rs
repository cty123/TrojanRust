use crate::protocol::common::addr::IpAddress;
use crate::protocol::common::atype::Atype;
use crate::protocol::common::command::Command;

#[derive(Debug, Copy, Clone)]
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
    pub fn addr_port(&self) -> (String, u16) {
        (self.addr.to_string(), self.port)
    }
}
