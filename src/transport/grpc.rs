tonic::include_proto!("trojan_rust.transport.grpc");

use crate::protocol::common::stream::StandardTcpStream;
use crate::proxy::grpc::server::{handle_client_data, handle_server_data};
use crate::transport::grpc::proxy_service_server::ProxyService;

use futures::Stream;
use log::{error, info};
use std::pin::Pin;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tonic::Request;
use tonic::Response;
use tonic::{self, Status, Streaming};

// TODO: Need more discretion in detemining the value for channel size, or make it configurable 
const CHANNEL_SIZE: usize = 128;

pub struct GrpcService();

impl GrpcService {
    pub fn new() -> Self {
        GrpcService {}
    }
}

#[tonic::async_trait]
impl ProxyService for GrpcService {
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
