use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncWrite};

#[async_trait]
pub trait IOStream: AsyncRead + AsyncWrite + Unpin + Send + Sync {}

#[async_trait]
pub trait InboundStream: AsyncRead + AsyncWrite + Unpin + Send + Sync {}

#[async_trait]
pub trait OutboundStream: AsyncRead + AsyncWrite + Unpin + Send + Sync {}

#[async_trait]
pub trait PacketStream: AsyncRead + AsyncWrite + Unpin + Send + Sync {}
