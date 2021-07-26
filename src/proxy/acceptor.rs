use std::io::{Error, ErrorKind, Result};

use tokio::io::{AsyncRead, AsyncWrite};
use tokio_rustls::TlsAcceptor;

use crate::protocol::common::stream::InboundStream;
use crate::protocol::socks5::inbound::Socks5InboundStream;
use crate::protocol::trojan::inbound::TrojanInboundStream;
use crate::proxy::base::SupportedProtocols;

#[derive(Clone)]
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

    pub async fn accept<IO>(self, inbound_stream: IO) -> Result<Box<dyn InboundStream>>
    where
        IO: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
    {
        match self.protocol {
            SupportedProtocols::SOCKS if self.tls_acceptor.is_some() => {
                let tls_stream = self.tls_acceptor.unwrap().accept(inbound_stream).await?;
                Ok(Socks5InboundStream::new(tls_stream, self.port))
            }
            SupportedProtocols::SOCKS => Ok(Socks5InboundStream::new(inbound_stream, self.port)),
            SupportedProtocols::TROJAN if self.tls_acceptor.is_some() => {
                let tls_stream = self.tls_acceptor.unwrap().accept(inbound_stream).await?;
                Ok(TrojanInboundStream::new(tls_stream))
            }
            SupportedProtocols::TROJAN => Ok(TrojanInboundStream::new(inbound_stream)),
            _ => Err(Error::new(
                ErrorKind::ConnectionReset,
                "Failed to accept inbound stream, unsupported protocol",
            )),
        }
    }
}
