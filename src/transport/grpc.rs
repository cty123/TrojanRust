tonic::include_proto!("trojan_rust.transport.grpc");

use crate::protocol::common::stream::StandardTcpStream;
use crate::protocol::trojan;
use crate::proxy::grpc::server::{handle_client_data, handle_server_data};
use crate::transport::grpc::grpc_service_server::GrpcService;
use crate::transport::grpc::proxy_service_server::ProxyService;

use bytes::{Buf, BufMut, BytesMut};
use futures::Stream;
use log::{error, info};
use std::io;
use std::pin::Pin;
use std::task::Poll;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{self, Sender};
use tonic::Request;
use tonic::Response;
use tonic::{self, Status, Streaming};

// TODO: Need more discretion in detemining the value for channel size, or make it configurable
const CHANNEL_SIZE: usize = 16;

pub struct TrojanProxyService();

impl TrojanProxyService {
    pub fn new() -> Self {
        Self {}
    }
}

#[tonic::async_trait]
impl ProxyService for TrojanProxyService {
    type ProxyStream = Pin<Box<dyn Stream<Item = Result<GrpcPacket, Status>> + Send>>;

    async fn proxy(
        &self,
        request: Request<Streaming<GrpcPacket>>,
    ) -> Result<Response<Self::ProxyStream>, Status> {
        let mut client_reader = request.into_inner();

        // We require the first GRPC message to be the proxy request itself. The proxy request message needs to
        // contain the information regarding to the request, such as secret, remote host name, etc. And if we
        // fail to read or parse the first message, we simply close the connection.
        let request = match client_reader.message().await {
            Ok(res) => match res {
                Some(r) => r,
                None => {
                    return Err(Status::aborted(
                        "received empty initial proxy request message",
                    ))
                }
            },
            Err(_) => {
                return Err(Status::aborted(
                    "failed to read initial proxy request message",
                ))
            }
        };

        // Create the channel used to write back the response message to the client
        let (tx, rx) = mpsc::channel(CHANNEL_SIZE);

        tokio::spawn(async move {
            // TODO: Support more protocols than just Trojan
            if let Some(trojan) = request.trojan {
                // Move out address and port from proxy request
                let (address, port) = (trojan.address, trojan.port);

                // Establish connection to remote server as specified by proxy request
                let (server_reader, server_writer) =
                    match TcpStream::connect((address.as_ref(), port as u16)).await {
                        Ok(stream) => tokio::io::split(StandardTcpStream::Plain(stream)),
                        Err(e) => {
                            error!("Failed to connect to {}:{}", address, port);
                            return Err(e);
                        }
                    };

                // Spawn two concurrent coroutines to transport the data between client and server
                tokio::select!(
                    _ = tokio::spawn(handle_server_data(client_reader, server_writer)) => (),
                    _ = tokio::spawn(handle_client_data(tx, server_reader)) => ()
                );

                info!("Connection finished");
            }

            Ok(())
        });

        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx),
        )))
    }
}

pub struct GrpcProxyService();

impl GrpcProxyService {
    pub fn new() -> Self {
        Self {}
    }
}

impl AsRef<[u8]> for Hunk {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

struct GrpcDataStream<T> {
    reader: Streaming<T>,
    buf: BytesMut,
}

impl<T> GrpcDataStream<T> {
    pub fn from_reader(reader: Streaming<T>) -> Self {
        Self {
            reader,
            buf: BytesMut::new(),
        }
    }
}

impl AsyncRead for GrpcDataStream<Hunk> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        // Used to indicate if we have new data available
        let mut has_new_data = false;

        // Check if the internal buffer has any data left
        if self.buf.has_remaining() {
            has_new_data = true;

            // Check if read buffer has enough space left
            if self.buf.remaining() <= buf.remaining() {
                // Dump the entire buffer into read buffer
                buf.put_slice(&self.buf);

                // Empty internal buffer
                self.buf.clear();
            } else {
                // Fill read buffer as much as we can
                let read_len = buf.remaining();
                buf.put_slice(&self.buf[..read_len]);

                // Advance internal buffer
                self.buf.advance(read_len);

                // Return as we have depleted read buffer
                return Poll::Ready(Ok(()));
            }
        }

        let data = match Pin::new(&mut self.reader).poll_next(cx) {
            Poll::Ready(d) => d,
            Poll::Pending if has_new_data => return Poll::Ready(Ok(())),
            Poll::Pending => return Poll::Pending,
        };

        let packet = match data {
            None => {
                return Poll::Ready(Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "Failed to read",
                )))
            }
            Some(packet) => match packet {
                Ok(p) => p,
                Err(_) => {
                    return Poll::Ready(Err(io::Error::new(
                        io::ErrorKind::BrokenPipe,
                        "Failed to read",
                    )))
                }
            },
        };

        // Check if the buffer is able to fit the packet
        if buf.remaining() >= packet.data.len() {
            // Write the entire packet to buffer if the buffer is large enough to fit
            buf.put_slice(&packet.data);
        } else {
            // Fill the read buffer as much as possible
            let rem = buf.remaining();
            buf.put_slice(&packet.data[..rem]);

            // Move the rest of the packet to internal buffer
            self.buf.put_slice(&packet.data[rem..]);
        }

        return Poll::Ready(Ok(()));
    }
}

#[tonic::async_trait]
impl GrpcService for GrpcProxyService {
    type TunStream = Pin<Box<dyn Stream<Item = Result<Hunk, Status>> + Send>>;
    type TunMultiStream = Pin<Box<dyn Stream<Item = Result<MultiHunk, Status>> + Send>>;

    async fn tun(
        &self,
        request: Request<Streaming<Hunk>>,
    ) -> Result<Response<Self::TunStream>, Status> {
        info!("Received GRPC request");

        let client_reader = request.into_inner();

        let (tx, rx) = mpsc::channel(CHANNEL_SIZE);

        tokio::spawn(async move {
            let mut stream = GrpcDataStream::from_reader(client_reader);

            // TODO: Support more protocols than just Trojan
            let request = trojan::parse(&mut stream).await?.inbound_request();

            // Move out address and port from proxy request
            let (address, port) = (request.addr, request.port);

            // Establish connection to remote server as specified by proxy request
            let (mut server_reader, mut server_writer) =
                match TcpStream::connect((address.to_string(), port as u16)).await {
                    Ok(stream) => tokio::io::split(stream),
                    Err(e) => {
                        error!("Failed to connect to {}:{}", address, port);
                        return Err(e);
                    }
                };

            // Spawn two concurrent coroutines to transport the data between client and server
            tokio::select!(
                _ = tokio::io::copy(&mut stream, &mut server_writer) => (),
                _ = write_back_traffic(&mut server_reader, tx) => (),
            );

            info!("Connection finished");
            Ok(())
        });

        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx),
        )))
    }

    async fn tun_multi(
        &self,
        request: Request<Streaming<MultiHunk>>,
    ) -> Result<Response<Self::TunMultiStream>, Status> {
        let mut client_reader = request.into_inner();

        // client_reader.into_async_read();

        let (tx, rx) = mpsc::channel(CHANNEL_SIZE);

        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx),
        )))
    }
}

async fn write_back_traffic<R: AsyncRead + Unpin>(
    mut reader: R,
    writer: Sender<Result<Hunk, Status>>,
) -> io::Result<()> {
    loop {
        let mut buf = Vec::with_capacity(4096);

        match reader.read_buf(&mut buf).await {
            Ok(_) => (),
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "Failed to read data from remote server",
                ))
            }
        }

        match writer.send(Ok(Hunk { data: buf })).await {
            Ok(_) => (),
            Err(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "Failed to send data to client",
                ))
            }
        }
    }
}
