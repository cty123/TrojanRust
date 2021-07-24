use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpStream, UdpSocket};

use crate::protocol::common::stream::{InboundStream, OutboundStream};
use crate::protocol::direct::outbound::DirectOutboundStream;
use crate::protocol::socks5::inbound::Socks5InboundStream;
use crate::proxy::base::SupportedProtocols;

pub struct ConnectionBuilder {
    protocol: SupportedProtocols,
    tls: bool,
}

impl ConnectionBuilder {
    pub fn new(protocol: SupportedProtocols, tls: bool) -> ConnectionBuilder {
        ConnectionBuilder {
            protocol: protocol,
            tls: tls,
        }
    }

    pub fn build_inbound<IO>(
        self,
        inbound_stream: IO,
        inbound_port: u16,
    ) -> Option<Box<dyn InboundStream>>
    where
        IO: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
    {
        match self.protocol {
            SupportedProtocols::SOCKS => {
                Some(Socks5InboundStream::new(inbound_stream, inbound_port))
            }
            _ => None,
        }
    }

    pub async fn build_outbound(self, request: String) -> Option<Box<dyn OutboundStream>> {
        match self.protocol {
            SupportedProtocols::DIRECT => {
                let connection = match TcpStream::connect(request).await {
                    Ok(connection) => connection,
                    Err(_) => return None,
                };
                Some(DirectOutboundStream::new(connection))
            }
            _ => None,
        }
    }
}
