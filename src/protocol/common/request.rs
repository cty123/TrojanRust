use crate::protocol::common::addr::IpAddress;

#[derive(Debug, Copy, Clone)]
pub enum TransportProtocol {
    TCP,
    UDP,
}

pub struct InboundRequest {
    pub atype: u8,
    pub addr: IpAddress,
    pub command: u8,
    pub port: u16,
    pub transport_protocol: TransportProtocol,
}

impl InboundRequest {
    #[inline]
    pub fn new(
        atype: u8,
        addr: IpAddress,
        command: u8,
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
