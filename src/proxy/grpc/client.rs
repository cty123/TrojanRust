use crate::protocol::common::stream::StandardTcpStream;
use crate::transport::grpc::GrpcPacket;
use std::io::{self, Error, ErrorKind};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::sync::mpsc::Sender;
use tonic::Streaming;

const BUFFER_SIZE: usize = 4096;

pub async fn handle_client_data<T: AsyncRead + AsyncWrite + Unpin>(
    mut client_writer: WriteHalf<StandardTcpStream<T>>,
    mut server_reader: Streaming<GrpcPacket>,
) -> io::Result<()> {
    loop {
        // Read response message from server
        let message = match server_reader.message().await {
            Ok(res) => match res {
                Some(packet) => packet,
                // TODO: We simply assume that the server response will always be non-empty
                None => continue,
            },
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::ConnectionReset,
                    "Failed to receive message",
                ))
            }
        };

        match message.datagram {
            Some(vec) => client_writer.write_all(&vec).await?,
            None => continue,
        };
    }
}

pub async fn handle_server_data<T: AsyncRead + AsyncWrite + Unpin>(
    mut client_reader: ReadHalf<StandardTcpStream<T>>,
    server_writer: Sender<GrpcPacket>,
) -> io::Result<()> {
    loop {
        let mut buf = Vec::with_capacity(BUFFER_SIZE);
        let n = client_reader.read_buf(&mut buf).await?;

        if n == 0 {
            return Ok(());
        }

        buf.truncate(n);

        match server_writer
            .send(GrpcPacket {
                packet_type: 0,
                trojan: None,
                datagram: Some(buf),
            })
            .await
        {
            Ok(_) => continue,
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::ConnectionRefused,
                    "Failed to receive message",
                ));
            }
        }
    }
}
