use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;

use crate::protocol::common::request::InboundRequest;
use crate::protocol::common::stream::OutboundStream;
use crate::protocol::direct::outbound::DirectOutboundStream;
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

    pub async fn handle(self, request: InboundRequest) -> Option<Box<dyn OutboundStream>> {
        match self.protocol {
            SupportedProtocols::DIRECT => {
                let connection = match TcpStream::connect(request.addr_port()).await {
                    Ok(connection) => connection,
                    Err(_) => return None,
                };
                Some(DirectOutboundStream::new(connection))
            }
            _ => None,
        }
    }
}
