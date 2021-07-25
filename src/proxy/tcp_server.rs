use log::{info, warn};

use std::io::{Error, ErrorKind, Result};

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpListener;

use crate::config::base::{InboundConfig, OutboundConfig};
use crate::config::tls::{load_certs, load_private_key};
use crate::proxy::acceptor::Acceptor;
use crate::proxy::handler::Handler;

pub struct TcpServer {
    local_port: u16,
    local_addr: String,
    inbound_tls: bool,
    outbound_tls: bool,
    acceptor: Acceptor,
    handler: Handler,
}

impl TcpServer {
    pub fn new(inbound: InboundConfig, outbound: OutboundConfig) -> Result<TcpServer> {
        let acceptor = Acceptor::new(inbound.protocol);
        let handler = Handler::new();

        let certificates = load_certs(&inbound.cert_path.unwrap())?;
        let key = load_private_key(&inbound.key_path.unwrap())?;

        return TcpServer {
            local_port: inbound.port,
            local_addr: inbound.address,
            acceptor,
            handler,
        };
    }

    pub async fn start(&self) -> Result<()> {
        let listener =
            TcpListener::bind(format!("{}:{}", self.local_addr, self.local_port)).await?;

        let local_port = self.local_port;

        info!(
            "TCP server started on port {}, ready to accept input stream",
            local_port
        );

        loop {
            let (socket, _) = listener.accept().await?;

            let acceptor = self.acceptor.clone();
            let handler = self.handler.clone();

            tokio::spawn(async move {
                match TcpServer::dispatch(socket, local_port, acceptor, handler).await {
                    Ok(()) => (),
                    Err(e) => warn!("Failed to handle the inbound stream: {}", e),
                }
            });
        }
    }

    async fn dispatch<IO>(
        inbound_stream: IO,
        inbound_port: u16,
        inbound_acceptor: Acceptor,
        outbound_handler: Handler,
    ) -> Result<()>
    where
        IO: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
    {
        let mut inbound_stream = match inbound_acceptor.accept(inbound_stream, inbound_port) {
            Some(stream) => stream,
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Unable to accept the inbound stream",
                ))
            }
        };

        let request = inbound_stream.handshake().await?;

        let outbound_stream = match outbound_handler.handle(request).await {
            Some(stream) => stream,
            None => {
                return Err(Error::new(
                    ErrorKind::ConnectionReset,
                    "Unable to establish connection to remote",
                ))
            }
        };

        let (mut source_read, mut source_write) = tokio::io::split(inbound_stream);
        let (mut target_read, mut target_write) = tokio::io::split(outbound_stream);

        return match futures::future::join(
            tokio::io::copy(&mut source_read, &mut target_write),
            tokio::io::copy(&mut target_read, &mut source_write),
        )
        .await
        {
            (Err(e), _) | (_, Err(e)) => Err(e),
            _ => Ok(()),
        };
    }
}
