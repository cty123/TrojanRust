use log::{info, warn};

use std::io::{Result, Error, ErrorKind};

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};
use tokio::net::TcpListener;

use crate::proxy::base::SupportedProtocols;
use crate::proxy::builder::ConnectionBuilder;

pub struct TcpServer {
    local_port: u16,
    local_addr: String,
    inbound_protocol: SupportedProtocols,
    outbound_protocol: SupportedProtocols,
}

impl TcpServer {
    pub fn new(
        local_port: u16,
        local_addr: String,
        inbound_protocol: SupportedProtocols,
        outbound_protocol: SupportedProtocols,
    ) -> TcpServer {
        return TcpServer {
            local_port,
            local_addr,
            inbound_protocol,
            outbound_protocol,
        };
    }

    pub async fn start(&self) -> Result<()> {
        let local_port = self.local_port;
        let listener = TcpListener::bind(format!("0.0.0.0:{}", local_port)).await?;

        info!(
            "TCP server started on port {}, ready to accept input stream",
            local_port
        );

        loop {
            let (socket, _) = listener.accept().await?;

            let inbound_builder = ConnectionBuilder::new(self.inbound_protocol, false);
            let outbound_builder = ConnectionBuilder::new(self.outbound_protocol, false);

            tokio::spawn(async move {
                match TcpServer::dispatch(socket, local_port, inbound_builder, outbound_builder)
                    .await
                {
                    Ok(()) => (),
                    Err(e) => warn!("Failed to handle the inbound stream: {}", e),
                }
            });
        }

        info!("TCP server exiting");

        Ok(())
    }

    async fn dispatch<IO>(
        inbound_stream: IO,
        inbound_port: u16,
        inbound_builder: ConnectionBuilder,
        outbound_builder: ConnectionBuilder,
    ) -> Result<()>
    where
        IO: AsyncRead + AsyncWrite + Unpin + Send + Sync + 'static,
    {

        let mut inbound_stream = match inbound_builder.build_inbound(inbound_stream, inbound_port) {
            Some(stream) => stream,
            None => return Err(Error::new(ErrorKind::InvalidInput, "Unsupported command"))
        };

        let request = inbound_stream.handshake().await?;

        let outbound_stream = match outbound_builder.build_outbound(request).await {
            Some(stream) => stream,
            None => return Err(Error::new(ErrorKind::InvalidInput, "Unsupported command"))
        };
        

        // let outbound_stream = match outbound_builder.build_outbound(outbound_stream, outbound_port)
        // let mut inbound_stream = Socks5InboundStream::new(inbound_stream, inbound_port);
        // let request = inbound_stream.handshake().await?;

        // let connection = TcpStream::connect(&request.request_addr_port()).await?;
        // let outbound_stream = DirectStream::new(connection);

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
        Ok(())
    }
}
