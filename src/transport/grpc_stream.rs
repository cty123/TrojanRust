use crate::transport::grpc_transport::Hunk;

use bytes::{Buf, BufMut, BytesMut};
use futures::ready;
use futures::Stream;
use std::io;
use std::pin::Pin;
use std::task::Poll;
use tokio::io::ReadBuf;
use tokio::io::AsyncRead;
use tonic::Status;
use tonic::{self, Streaming};

pub struct GrpcDataReaderStream<T> {
    reader: Streaming<T>,
    buf: BytesMut,
}

impl<T> GrpcDataReaderStream<T> {
    #[inline]
    pub fn from_reader(reader: Streaming<T>) -> Self {
        Self {
            reader,
            buf: BytesMut::new(),
        }
    }
}

impl AsyncRead for GrpcDataReaderStream<Hunk> {
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

pub struct GrpcHunkRequestStream<T> {
    inner: T,
}

impl<T> GrpcHunkRequestStream<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

pub struct GrpcHunkResponseStream<T> {
    inner: T,
}

impl<T> GrpcHunkResponseStream<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T> Stream for GrpcHunkRequestStream<T>
where
    T: AsyncRead + Unpin,
{
    type Item = Hunk;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut buf = vec![0; 4096];

        let mut read_buf = ReadBuf::new(&mut buf);

        match ready!(Pin::new(&mut self.inner).poll_read(cx, &mut read_buf)) {
            Ok(_) => (),
            Err(_) => return Poll::Ready(None),
        }

        let size = read_buf.filled().len();

        buf.truncate(size);

        return Poll::Ready(Some(Hunk { data: buf }));
    }
}

impl<T> Stream for GrpcHunkResponseStream<T>
where
    T: AsyncRead + Unpin,
{
    type Item = Result<Hunk, Status>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut buf = vec![0; 4096];

        let mut read_buf = ReadBuf::new(&mut buf);

        match ready!(Pin::new(&mut self.inner).poll_read(cx, &mut read_buf)) {
            Ok(_) => (),
            Err(_) => return Poll::Ready(Some(Err(Status::aborted("Failed to poll the data")))),
        }

        let size = read_buf.filled().len();

        buf.truncate(size);

        return Poll::Ready(Some(Ok(Hunk { data: buf })));
    }
}
