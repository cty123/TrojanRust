use crate::transport::grpc_transport::{Hunk, MultiHunk};

use bytes::{Buf, BufMut, BytesMut};
use futures::Stream;
use std::io;
use std::pin::Pin;
use std::task::Poll;
use tokio::{io::AsyncRead, io::AsyncWrite, sync::mpsc::Sender};
use tonic::{self, Status, Streaming};

pub struct GrpcDataStream<T> {
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

// pub struct GrpcWriterDataStream {
//     inner: Sender<Result<Hunk, Status>>,
//     buffer: BytesMut,
// }

// impl AsyncWrite for GrpcWriterDataStream {
//     fn poll_write(
//         self: Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//         buf: &[u8],
//     ) -> Poll<Result<usize, io::Error>> {
//         self.buffer.put_slice(buf);

//         return Poll::Ready(Ok(buf.len()));
//     }

//     fn poll_flush(
//         self: Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//     ) -> Poll<Result<(), io::Error>> {
//         if self.buffer.remaining() > 0 {
//             self.inner.try_send(message)
//         }
//     }

//     fn poll_shutdown(
//         self: Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//     ) -> Poll<Result<(), io::Error>> {
//         todo!()
//     }
// }
