use std::io::Result;

use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::protocol::common::request::InboundRequest;

#[async_trait]
pub trait IOStream: AsyncRead + AsyncWrite + Unpin + Send + Sync {}

#[async_trait]
pub trait InboundStream: AsyncRead + AsyncWrite + Unpin + Send + Sync {
    async fn handshake(&mut self) -> Result<InboundRequest>;
}

#[async_trait]
pub trait OutboundStream: AsyncRead + AsyncWrite + Unpin + Send + Sync {}
