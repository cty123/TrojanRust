use crate::{
    config::base::InboundConfig,
    protocol::{common::request::InboundRequest, trojan},
    proxy::base::SupportedProtocols,
    transport::{grpc_stream::GrpcDataStream, grpc_transport::Hunk},
};

use once_cell::sync::OnceCell;
use sha2::{Digest, Sha224};
use std::io::{self, Error, ErrorKind};
use tokio::sync::mpsc::Sender;
use tonic::{Request, Status, Streaming};

/// Static lifetime GRPC acceptor
static GRPC_ACCEPTOR: OnceCell<GrpcAcceptor> = OnceCell::new();

/// Acceptor handles incomming connection by escalating them to application level data stream based on
/// the configuration. It is also responsible for escalating TCP connection to TLS connection if the user
/// enabled TLS.
pub struct GrpcAcceptor {
    protocol: SupportedProtocols,
    secret: Vec<u8>,
}

impl GrpcAcceptor {
    pub fn new(inbound_config: &InboundConfig) -> &'static GrpcAcceptor {
        let secret = match inbound_config.protocol {
            SupportedProtocols::TROJAN if inbound_config.secret.is_some() => {
                let secret = inbound_config.secret.as_ref().unwrap();
                Sha224::digest(secret.as_bytes())
                    .iter()
                    .map(|x| format!("{:02x}", x))
                    .collect::<String>()
                    .as_bytes()
                    .to_vec()
            }
            _ => Vec::new(),
        };

        GRPC_ACCEPTOR.get_or_init(|| Self {
            protocol: inbound_config.protocol,
            secret,
        })
    }

    pub async fn accept_hunk(
        &self,
        request: Request<Streaming<Hunk>>,
    ) -> io::Result<(InboundRequest, GrpcDataStream<Hunk>)> {
        // Convert request into inbound reader stream
        let mut inbound_reader = GrpcDataStream::from_reader(request.into_inner());

        // Based on the protocol, decide how to proceed with the inbound stream
        // TODO: Support more protocols than just Trojan
        let request = match self.protocol {
            SupportedProtocols::TROJAN => {
                let trojan_request = trojan::parse(&mut inbound_reader).await?;
                if !trojan_request.validate(&self.secret) {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "Incorrect trojan credentials",
                    ));
                }

                trojan_request.inbound_request()
            }
            _ => return Err(Error::new(ErrorKind::Unsupported, "Unsupported protocol")),
        };

        Ok((request, inbound_reader))
    }
}
