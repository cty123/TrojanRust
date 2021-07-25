use std::io::{Error, ErrorKind, Result};

use tokio::io::{AsyncRead, AsyncWrite};
use tokio_rustls::TlsAcceptor;

use crate::protocol::common::stream::InboundStream;
use crate::protocol::socks5::inbound::Socks5InboundStream;
use crate::proxy::base::SupportedProtocols;

pub struct Acceptor {
    tls_acceptor: Option<TlsAcceptor>,
    port: u16,
    protocol: SupportedProtocols,
}

impl Acceptor {
    pub fn new(
        protocol: SupportedProtocols,
        port: u16,
        tls_acceptor: Option<TlsAcceptor>,
    ) -> Acceptor {
        Acceptor {
            tls_acceptor,
            port,
            protocol,
        }
    }

    pub async fn accept<IO>(&self, inbound_stream: IO) -> Result<Box<dyn InboundStream>>
    where
        IO: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
    {
        match self.protocol {
            SupportedProtocols::SOCKS => Ok(Socks5InboundStream::new(inbound_stream, self.port)),
            _ => Err(Error::new(
                ErrorKind::ConnectionReset,
                "Failed to accept inbound stream, unsupported protocol",
            )),
        }
    }
}
