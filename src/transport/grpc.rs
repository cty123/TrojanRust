use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::{Buf, BufMut, BytesMut};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::sync::mpsc::Sender;
use tokio_stream;
use tokio_stream::Stream;
use tonic::{Status, Streaming};

tonic::include_proto!("trojan_rust.transport.grpc");

pub struct GrpcDataInboundStream {
    read_half: Streaming<GrpcDatagram>,
    write_half: Sender<Result<GrpcDatagram, Status>>,
    buffer: BytesMut,
}

impl GrpcDataInboundStream {
    pub fn new(
        read_half: Streaming<GrpcDatagram>,
        write_half: Sender<Result<GrpcDatagram, Status>>,
    ) -> Self {
        GrpcDataInboundStream {
            read_half,
            write_half,
            buffer: BytesMut::new(),
        }
    }
}

impl AsyncRead for GrpcDataInboundStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let initial_remaining = buf.remaining();

        // If the read buffer is able to store all the data in the internal buffer
        if !self.buffer.is_empty() {
            if buf.remaining() >= self.buffer.len() {
                // Dump everything from internal buffer to read buffer if the it is large enough
                buf.put_slice(&self.buffer);
                self.buffer.clear();
            } else {
                // Otherwise take as many as possible and advance internal buffer position
                let size = buf.remaining();
                buf.put_slice(&self.buffer[..size]);
                self.buffer.advance(size);

                return Poll::Ready(Ok(()));
            }
        }

        // Keep reading until the buffer is full
        while buf.remaining() > 0 {
            match Pin::new(&mut self.read_half).poll_next(cx) {
                Poll::Pending => {
                    if initial_remaining > buf.remaining() {
                        return Poll::Ready(Ok(()));
                    } else {
                        return Poll::Pending;
                    }
                }
                Poll::Ready(res) => {
                    match res {
                        Some(result) => match result {
                            Ok(datagram) => {
                                if buf.remaining() >= datagram.payload.len() {
                                    // Directly write to read buffer if it has adequate space
                                    buf.put_slice(&datagram.payload);
                                } else {
                                    // Otherwise fill in as many as possible and dump the rest into internal buffer
                                    let size = buf.remaining();
                                    buf.put_slice(&datagram.payload[..size]);
                                    self.buffer.put_slice(&datagram.payload[size..]);

                                    // Since we already run out of read buffer, there's no need to continue
                                    return Poll::Ready(Ok(()));
                                }
                            }
                            Err(err) => {
                                return Poll::Ready(Err(io::Error::new(
                                    io::ErrorKind::InvalidData,
                                    err.to_string(),
                                )))
                            }
                        },
                        None => {
                            return Poll::Ready(Err(io::Error::new(
                                io::ErrorKind::ConnectionReset,
                                "finished reading",
                            )));
                        }
                    }
                }
            };
        }

        Poll::Ready(Ok(()))
    }
}

impl AsyncWrite for GrpcDataInboundStream {
    #[inline]
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        // TODO: Boost write performance by using buffer.
        return match self.write_half.try_send(Ok(GrpcDatagram {
            payload: buf.to_vec(),
        })) {
            Ok(_) => Poll::Ready(Ok(buf.len())),
            Err(e) => Poll::Ready(Err(io::Error::new(
                io::ErrorKind::Interrupted,
                e.to_string(),
            ))),
        };
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    #[inline]
    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

unsafe impl Send for GrpcDataInboundStream {}
unsafe impl Sync for GrpcDataInboundStream {}

pub struct GrpcDataOutboundStream {
    read_half: Streaming<GrpcDatagram>,
    write_half: Sender<GrpcDatagram>,
    buffer: BytesMut,
}

impl GrpcDataOutboundStream {
    pub fn new(read_half: Streaming<GrpcDatagram>, write_half: Sender<GrpcDatagram>) -> Self {
        Self {
            read_half,
            write_half,
            buffer: BytesMut::new(),
        }
    }
}

impl AsyncRead for GrpcDataOutboundStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let initial_remaining = buf.remaining();

        // If the read buffer is able to store all the data in the internal buffer
        if !self.buffer.is_empty() {
            if buf.remaining() >= self.buffer.len() {
                // Dump everything from internal buffer to read buffer if the it is large enough
                buf.put_slice(&self.buffer);
                self.buffer.clear();
            } else {
                // Otherwise take as many as possible and advance internal buffer position
                let size = buf.remaining();
                buf.put_slice(&self.buffer[..size]);
                self.buffer.advance(size);
                return Poll::Ready(Ok(()));
            }
        }

        // Keep reading until the buffer is full
        while buf.remaining() > 0 {
            match Pin::new(&mut self.read_half).poll_next(cx) {
                Poll::Pending => {
                    if initial_remaining > buf.remaining() {
                        return Poll::Ready(Ok(()));
                    } else {
                        return Poll::Pending;
                    }
                }
                Poll::Ready(res) => {
                    match res {
                        Some(result) => match result {
                            Ok(datagram) => {
                                if buf.remaining() >= datagram.payload.len() {
                                    // Directly write to read buffer if it has adequate space
                                    buf.put_slice(&datagram.payload);
                                } else {
                                    // Otherwise fill in as many as possible and dump the rest into internal buffer
                                    let size = buf.remaining();
                                    buf.put_slice(&datagram.payload[..size]);
                                    self.buffer.put_slice(&datagram.payload[size..]);

                                    // Since we already run out of read buffer, there's no need to continue
                                    return Poll::Ready(Ok(()));
                                }
                            }
                            Err(err) => {
                                return Poll::Ready(Err(io::Error::new(
                                    io::ErrorKind::InvalidData,
                                    err.to_string(),
                                )))
                            }
                        },
                        None => {
                            return Poll::Ready(Err(io::Error::new(
                                io::ErrorKind::ConnectionReset,
                                "finished reading",
                            )))
                        }
                    }
                }
            };
        }

        Poll::Ready(Ok(()))
    }
}

impl AsyncWrite for GrpcDataOutboundStream {
    #[inline]
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        // TODO: Boost write performance by using buffer.
        return match self.write_half.try_send(GrpcDatagram {
            payload: buf.to_vec(),
        }) {
            Ok(_) => Poll::Ready(Ok(buf.len())),
            Err(e) => Poll::Ready(Err(io::Error::new(io::ErrorKind::Interrupted, e))),
        };
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    #[inline]
    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

unsafe impl Send for GrpcDataOutboundStream {}
unsafe impl Sync for GrpcDataOutboundStream {}
