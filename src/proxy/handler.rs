use std::io::{Error, ErrorKind, Result};

use log::{info, warn};
use tokio::net::TcpStream;

use crate::protocol::common::request::{InboundRequest, TransportProtocol};
use crate::protocol::common::stream::OutboundStream;
use crate::protocol::direct::outbound::DirectOutboundStream;
use crate::protocol::trojan::outbound::TrojanOutboundStream;
use crate::protocol::trojan::packet::PacketTrojanOutboundStream;
use crate::proxy::base::SupportedProtocols;

#[derive(Clone)]
pub struct Handler {
    protocol: SupportedProtocols,
    tls: bool,
    destination: Option<String>,
}

impl Handler {
    pub fn new() -> Handler {
        Handler {
            protocol: SupportedProtocols::DIRECT,
            destination: None,
            tls: false,
        }
    }

    pub async fn handle(self, request: &InboundRequest) -> Result<Box<dyn OutboundStream>> {
        let addr_port = request.addr_port();

        return match request.transport_protocol {
            TransportProtocol::TCP => {
                let connection = match TcpStream::connect(&addr_port).await {
                    Ok(connection) => connection,
                    Err(e) => {
                        return Err(Error::new(
                            ErrorKind::ConnectionReset,
                            format!("Failed to connect to {}: {}", &addr_port, e),
                        ))
                    }
                };

                return match self.protocol {
                    SupportedProtocols::DIRECT => Ok(DirectOutboundStream::new(connection)),
                    SupportedProtocols::TROJAN => {
                        Ok(TrojanOutboundStream::new(connection, request))
                    }
                    _ => Err(Error::new(ErrorKind::InvalidInput, "Unsupported protocol")),
                };
            }
            TransportProtocol::UDP => {
                info!("UDP associate to {}", request.addr_port());
                match PacketTrojanOutboundStream::new().await {
                    Ok(c) => Ok(c),
                    Err(_) => Err(Error::new(
                        ErrorKind::ConnectionReset,
                        format!("Failed to dial udp connection to {}", addr_port),
                    )),
                }
            }
        };
    }
}
