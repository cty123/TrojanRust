use std::io::{Error, ErrorKind, Result};

use tokio::net::TcpStream;
use log::{info, warn};

use crate::protocol::common::request::InboundRequest;
use crate::protocol::common::request::TransportProtocol;
use crate::protocol::common::stream::OutboundStream;
use crate::protocol::direct::outbound::DirectOutboundStream;
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

    // pub fn () -> Handler {
    //     Handler {}
    // }

    pub async fn handle(self, request: &InboundRequest) -> Result<Box<dyn OutboundStream>> {
        let addr_port = request.addr_port();

        match self.protocol {
            SupportedProtocols::DIRECT => {
                return match request.transport_protocol() {
                    TransportProtocol::TCP => {
                        info!("Dialing to {}", request.addr_port());
                        let connection = match TcpStream::connect(&addr_port).await {
                            Ok(connection) => connection,
                            Err(e) => {
                                return Err(Error::new(
                                    ErrorKind::ConnectionReset,
                                    format!("Failed to connect to {}: {}", &addr_port, e),
                                ))
                            }
                        };
                        Ok(DirectOutboundStream::new(connection))
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
                }
            }
            _ => Err(Error::new(ErrorKind::InvalidInput, "Unsupported protocol")),
        }
    }
}
