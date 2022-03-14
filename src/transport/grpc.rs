use crate::protocol::common::stream::StandardTcpStream;
use crate::proxy::grpc::server::{handle_client_data, handle_server_data};
use futures::Stream;
use log::{error, info};
use std::pin::Pin;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tonic::Request;
use tonic::Response;
use tonic::{self, Status, Streaming};

tonic::include_proto!("trojan_rust.transport.grpc");

use crate::transport::grpc::proxy_service_server::ProxyService;

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

        let request = match client_reader.message().await {
            Ok(res) => match res {
                Some(r) => r,
                None => return Err(Status::aborted("failed to extract stream")),
            },
            Err(_) => return Err(Status::aborted("failed to read incoming message")),
        };

        let (mut tx, rx) = mpsc::channel(64);

        tokio::spawn(async move {
            if let Some(trojan) = request.trojan {
                let address = trojan.address;
                let port = trojan.port;
                let (mut server_reader, mut server_writer) =
                    match TcpStream::connect((address, port as u16)).await {
                        Ok(stream) => tokio::io::split(StandardTcpStream::Plain(stream)),
                        Err(e) => return Err(e),
                    };

                return match tokio::try_join!(
                    handle_server_data(&mut client_reader, &mut server_writer),
                    handle_client_data(&mut tx, &mut server_reader)
                ) {
                    Ok(_) => {
                        info!("Connection finished");
                        Ok(())
                    }
                    Err(e) => {
                        error!("Encountered {} error while handling the transport", e);
                        Err(e)
                    }
                };
            }

            Ok(())
        });

        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx),
        )))
    }
}
