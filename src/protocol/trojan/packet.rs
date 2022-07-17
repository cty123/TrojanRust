use std::net::IpAddr;
use std::sync::Arc;

use async_trait::async_trait;
use log::info;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::UdpSocket;

use crate::protocol::common::addr::IpAddress;
use crate::protocol::common::atype::Atype;
use crate::protocol::common::request::InboundRequest;
use crate::protocol::common::stream::{PacketReader, PacketWriter};

pub struct TrojanPacketReader<T> {
    inner: T,
}

pub struct TrojanPacketWriter<T> {
    inner: T,
    request: InboundRequest,
}

impl<T: AsyncRead + Unpin + Send> TrojanPacketReader<T> {
    #[inline]
    pub fn new(inner: T) -> Self {
        TrojanPacketReader { inner }
    }
}

impl<T: AsyncWrite + Unpin + Send> TrojanPacketWriter<T> {
    #[inline]
    pub fn new(inner: T, request: InboundRequest) -> Self {
        TrojanPacketWriter { inner, request }
    }
}

#[async_trait]
impl<T: AsyncRead + Unpin + Send> PacketReader for TrojanPacketReader<T> {
    async fn read(&mut self) -> std::io::Result<Vec<u8>> {
        // Read address type
        let atype = self.inner.read_u8().await?;

        // Read the address type
        match Atype::from(atype)? {
            Atype::IPv4 => {
                self.inner.read_u32().await?;
            }
            Atype::IPv6 => {
                self.inner.read_u128().await?;
            }
            Atype::DomainName => {
                // Get payload size
                let size = self.inner.read_u8().await? as usize;
                let mut buf = vec![0u8; size];

                // Read data into buffer
                self.inner.read_exact(&mut buf).await?;
            }
        };

        // Read port, payload length and CRLF
        let _port = self.inner.read_u16().await?;
        let length = self.inner.read_u16().await?;
        self.inner.read_u16().await?;

        // Read data into the buffer
        let mut buf = Vec::with_capacity(length as usize);
        self.inner.read_buf(&mut buf).await?;

        Ok(buf)
    }
}

#[async_trait]
impl<T: AsyncWrite + Unpin + Send> PacketWriter for TrojanPacketWriter<T> {
    async fn write(&mut self, buf: &[u8]) -> std::io::Result<()> {
        // Write address type to remote
        self.inner.write_u8(self.request.atype.to_byte()).await?;

        // Write back the address of the trojan request
        match self.request.addr {
            IpAddress::IpAddr(IpAddr::V4(addr)) => {
                self.inner.write_all(&addr.octets()).await?;
            }
            IpAddress::IpAddr(IpAddr::V6(addr)) => {
                self.inner.write_all(&addr.octets()).await?;
            }
            IpAddress::Domain(ref domain) => {
                self.inner.write_u8(domain.to_bytes().len() as u8).await?;
                self.inner.write_all(domain.to_bytes()).await?;
            }
        }

        // Write port, payload size, CRLF, and the payload data into the stream
        self.inner.write_u16(self.request.port).await?;
        self.inner.write_u16(buf.len() as u16).await?;
        self.inner.write_u16(0x0D0A).await?;
        self.inner.write_all(buf).await?;
        self.inner.flush().await?;
        Ok(())
    }
}

pub async fn packet_reader_to_stream_writer<R: AsyncRead + Unpin + Send, W: AsyncWrite + Unpin>(
    mut reader: TrojanPacketReader<R>,
    mut writer: W,
) -> std::io::Result<()> {
    loop {
        let buf = reader.read().await?;
        writer.write_all(&buf).await?;
        writer.flush().await?;
    }
}

pub async fn stream_reader_to_packet_writer<R: AsyncRead + Unpin, W: AsyncWrite + Unpin + Send>(
    mut reader: R,
    mut writer: TrojanPacketWriter<W>,
) -> std::io::Result<()> {
    loop {
        let mut buf = Vec::with_capacity(4096);
        reader.read_buf(&mut buf).await?;
        writer.write(&buf).await?;
    }
}

pub async fn packet_reader_to_udp_packet_writer<R: AsyncRead + Unpin + Send>(
    mut reader: TrojanPacketReader<R>,
    writer: Arc<UdpSocket>,
) -> std::io::Result<()> {
    loop {
        let data = reader.read().await?;
        writer.send(&data).await?;
    }
}

pub async fn udp_packet_reader_to_packet_writer<W: AsyncWrite + Unpin + Send>(
    reader: Arc<UdpSocket>,
    mut writer: TrojanPacketWriter<W>,
) -> std::io::Result<()> {
    loop {
        let mut buf = vec![0u8; 4096];
        reader.recv(&mut buf).await?;
        writer.write(&buf).await?;
    }
}
