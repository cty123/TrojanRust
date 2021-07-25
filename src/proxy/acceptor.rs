use tokio::io::{AsyncRead, AsyncWrite};

use crate::protocol::common::stream::InboundStream;
use crate::protocol::socks5::inbound::Socks5InboundStream;
use crate::proxy::base::SupportedProtocols;

#[derive(Clone)]
pub struct Acceptor {
    protocol: SupportedProtocols,
}

impl Acceptor {
    pub fn new(protocol: SupportedProtocols) -> Acceptor {
        Acceptor { protocol }
    }

    pub fn accept<IO>(self, inbound_stream: IO, inbound_port: u16) -> Option<Box<dyn InboundStream>>
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
}
