use std::io::{Error, ErrorKind, Result};

use log::info;
use sha2::{Digest, Sha224};
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
    secret: Vec<u8>,
}

impl Acceptor {
    pub fn new(
        protocol: SupportedProtocols,
        port: u16,
        tls_acceptor: Option<TlsAcceptor>,
        secret: Option<String>,
    ) -> Acceptor {
        let secret = match protocol {
            SupportedProtocols::TROJAN if secret.is_some() => {
                let secret = secret.unwrap();
                let hashvalue = Sha224::digest(secret.as_bytes());
                hashvalue
                    .iter()
                    .map(|x| format!("{:02x}", x))
                    .collect::<String>()
                    .as_bytes()
                    .to_vec()
            }
            _ => Vec::new(),
        };
        info!("Computed hash {:?}", secret);

        Acceptor {
            tls_acceptor,
            port,
            protocol,
            secret,
        }
    }

    pub async fn accept<IO>(self, inbound_stream: IO) -> Result<Box<dyn InboundStream>>
    where
        IO: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
    {
        match self.protocol {
            // Socks5 with or without TLS
            SupportedProtocols::SOCKS if self.tls_acceptor.is_some() => {
                let tls_stream = self.tls_acceptor.unwrap().accept(inbound_stream).await?;
                Ok(Socks5InboundStream::new(tls_stream, self.port))
            }
            SupportedProtocols::SOCKS => Ok(Socks5InboundStream::new(inbound_stream, self.port)),
            // Trojan wih or without TLS
            SupportedProtocols::TROJAN if self.tls_acceptor.is_some() => {
                let tls_stream = self.tls_acceptor.unwrap().accept(inbound_stream).await?;
                Ok(TrojanInboundStream::new(tls_stream, self.secret.as_slice()))
            }
            SupportedProtocols::TROJAN => Ok(TrojanInboundStream::new(
                inbound_stream,
                self.secret.as_slice(),
            )),
            _ => Err(Error::new(
                ErrorKind::ConnectionReset,
                "Failed to accept inbound stream, unsupported protocol",
            )),
        }
    }
}
