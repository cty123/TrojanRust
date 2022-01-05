use log::info;
use std::io;
use std::pin::Pin;
use std::sync::Arc;

use sha2::{Digest, Sha224};
use tokio::fs::read;
use tokio::sync::mpsc;
use tokio_stream;
use tokio_stream::Stream;
use tonic::transport::{Identity, Server, ServerTlsConfig};
use tonic::{Request, Response, Status, Streaming};

use crate::config::base::{InboundConfig, OutboundConfig};
use crate::protocol::trojan::inbound::TrojanInboundStream;
use crate::proxy::handler::Handler;
use crate::transport::grpc::proxy_service_server::ProxyService;
use crate::transport::grpc::proxy_service_server::ProxyServiceServer;
use crate::transport::grpc::{GrpcDataInboundStream, GrpcDatagram};

pub struct GrpcProxyService {
    secret: Arc<Vec<u8>>,
    handler: Arc<Handler>,
}

impl GrpcProxyService {
    pub fn new(secret: Vec<u8>, handler: Handler) -> Self {
        Self {
            secret: Arc::from(secret),
            handler: Arc::from(handler),
        }
    }
}

#[tonic::async_trait]
impl ProxyService for GrpcProxyService {
    type ProxyStream =
        Pin<Box<dyn Stream<Item = Result<GrpcDatagram, Status>> + Send + Sync + 'static>>;

    async fn proxy(
        &self,
        request: Request<Streaming<GrpcDatagram>>,
    ) -> Result<Response<Self::ProxyStream>, Status> {
        let (tx, rx) = mpsc::channel(64);

        let handler = self.handler.clone();
        let secret = self.secret.clone();

        tokio::spawn(async move {
            let (request, inbound_stream) = match TrojanInboundStream::new(
                GrpcDataInboundStream::new(request.into_inner(), tx),
                &secret,
            )
            .await
            {
                Ok(result) => result,
                Err(e) => {
                    info!("Error: {}", e.to_string());
                    return Err(Status::invalid_argument(e.to_string()));
                }
            };

            return match handler.dispatch(inbound_stream, request).await {
                Ok(_) => Ok(()),
                Err(e) => Err(Status::aborted(e.to_string())),
            };
        });

        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx),
        )))
    }
}

pub struct GrpcServer {
    local_addr: String,
    local_port: u16,
    service: GrpcProxyService,
    identity: Option<Identity>,
}

impl GrpcServer {
    pub async fn new(
        inbound_config: InboundConfig,
        outbound_config: OutboundConfig,
    ) -> io::Result<Self> {
        let secret = Sha224::digest(inbound_config.secret.as_ref().unwrap().as_bytes())
            .iter()
            .map(|x| format!("{:02x}", x))
            .collect::<String>()
            .as_bytes()
            .to_vec();

        let service = GrpcProxyService::new(secret, Handler::new(outbound_config)?);

        let identity = match inbound_config.tls {
            Some(tls_config) => {
                let cert = read(tls_config.cert_path).await?;
                let key = read(tls_config.key_path).await?;
                Some(Identity::from_pem(cert, key))
            }
            None => None,
        };

        Ok(Self {
            local_addr: inbound_config.address,
            local_port: inbound_config.port,
            service,
            identity,
        })
    }

    pub async fn start(self) -> io::Result<()> {
        info!(
            "GRPC server started on {}:{}, ready to accept input stream",
            self.local_addr, self.local_port
        );

        let addr = format!("{}:{}", self.local_addr, self.local_port)
            .parse()
            .unwrap();

        let mut server = match self.identity {
            Some(identity) => Server::builder()
                .tls_config(ServerTlsConfig::new().identity(identity))
                .unwrap(),
            None => Server::builder(),
        };

        return match server
            .add_service(ProxyServiceServer::new(self.service))
            .serve(addr)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(io::Error::new(
                io::ErrorKind::AddrNotAvailable,
                e.to_string(),
            )),
        };
    }
}
