use std::io::{Error, ErrorKind, Result};
use std::sync::Arc;

use sha2::{Digest, Sha224};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_rustls::TlsAcceptor;

use crate::config::base::InboundConfig;
use crate::config::tls::make_server_config;
use crate::protocol::common::request::InboundRequest;
use crate::protocol::common::stream::InboundStream;
use crate::protocol::socks5::inbound::Socks5InboundStream;
use crate::protocol::trojan::inbound::TrojanInboundStream;
use crate::proxy::base::SupportedProtocols;

/// Acceptor handles incomming connection by escalating them to application level data stream based on
/// the configuration. It is also responsible for escalating TCP connection to TLS connection if the user
/// enabled TLS.
#[derive(Clone)]
pub struct Acceptor {
    tls_acceptor: Option<TlsAcceptor>,
    port: u16,
    protocol: SupportedProtocols,
    secret: Arc<Vec<u8>>,
}

impl Acceptor {
    /// Instantiate a new acceptor based on InboundConfig passed by the user. It will generate the secret based on
    /// secret in the config file and the selected protocol and instantiate TLS acceptor is it is enabled.
    pub fn new(inbound: &InboundConfig) -> Acceptor {
        let secret = match inbound.protocol {
            SupportedProtocols::TROJAN if inbound.secret.is_some() => {
                let secret = inbound.secret.as_ref().unwrap();
                Sha224::digest(secret.as_bytes())
                    .iter()
                    .map(|x| format!("{:02x}", x))
                    .collect::<String>()
                    .as_bytes()
                    .to_vec()
            }
            _ => Vec::new(),
        };

        let tls_acceptor = match &inbound.tls {
            Some(tls) => match make_server_config(&tls) {
                Some(cfg) => Some(TlsAcceptor::from(cfg)),
                None => None,
            },
            None => None,
        };

        Acceptor {
            tls_acceptor,
            port: inbound.port,
            protocol: inbound.protocol,
            secret: Arc::from(secret),
        }
    }

    /// Takes an inbound TCP stream, escalate to TLs if possible and then escalate to application level data stream
    /// to be ready to read user's request and process them.
    pub async fn accept<IO>(
        &self,
        inbound_stream: IO,
    ) -> Result<(InboundRequest, Box<dyn InboundStream>)>
    where
        IO: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
    {
        match self.protocol {
            // Socks5 with or without TLS
            SupportedProtocols::SOCKS if self.tls_acceptor.is_some() => {
                let tls_stream = self
                    .tls_acceptor
                    .as_ref()
                    .unwrap()
                    .accept(inbound_stream)
                    .await?;
                Ok(Socks5InboundStream::new(tls_stream, self.port).await?)
            }
            SupportedProtocols::SOCKS => {
                Ok(Socks5InboundStream::new(inbound_stream, self.port).await?)
            }
            // Trojan wih or without TLS
            SupportedProtocols::TROJAN if self.tls_acceptor.is_some() => {
                let tls_stream = self
                    .tls_acceptor
                    .as_ref()
                    .unwrap()
                    .accept(inbound_stream)
                    .await?;
                Ok(TrojanInboundStream::new(tls_stream, &self.secret).await?)
            }
            SupportedProtocols::TROJAN => {
                Ok(TrojanInboundStream::new(inbound_stream, &self.secret).await?)
            }
            // Shutdown the connection if the protocol is currently unsupported
            _ => Err(Error::new(
                ErrorKind::ConnectionReset,
                "Failed to accept inbound stream, unsupported protocol",
            )),
        }
    }
}
