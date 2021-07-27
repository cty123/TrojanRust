use crate::protocol::common::addr::IpAddress;

#[derive(Debug)]
pub enum TransportProtocol {
    TCP,
    UDP,
}

pub struct InboundRequest {
    addr: IpAddress,
    port: u16,
    transport_protocol: TransportProtocol,
}

impl InboundRequest {
    #[inline]
    pub fn new(
        addr: IpAddress,
        port: u16,
        transport_protocol: TransportProtocol,
    ) -> InboundRequest {
        InboundRequest {
            addr,
            port,
            transport_protocol,
        }
    }

    #[inline]
    pub fn addr_port(self) -> String {
        format!("{}:{}", self.addr, self.port)
    }

    #[inline]
    pub fn transport_protocol(self) -> TransportProtocol {
        self.transport_protocol
    }
}
