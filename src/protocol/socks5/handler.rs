use log::{debug, error, info, warn};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_util::codec::{BytesCodec, FramedRead, FramedWrite};
use futures::{future, StreamExt, SinkExt};

use crate::protocol::socks5::base::{ServerHello, Request, RequestAck};
use crate::protocol::socks5::parser;

pub struct Handler {
    socket: TcpStream,
    request: Option<Request>,
    buf: [u8; 1024],
}

impl Handler {

    pub fn new(mut socket: TcpStream) -> Handler {
        Handler {
            socket,
            request: None,
            buf: [0; 1024],
        }
    }

    pub async fn handle(&mut self) -> Result<(), String> {
        match self.handle_handshake().await {
            Ok(_) => (),
            Err(e) => return Err(e)
        };

        self.request = Option::from(match self.handle_request().await {
            Ok(r) => r,
            Err(e) => return Err(e)
        });

        info!("Received socks request: {}", self.request.as_ref().unwrap().dump_request());

        match self.handle_request_ack().await {
            Ok(()) => (),
            Err(e) => return Err(e)
        }

        // Starts transporting data back and forth
        match self.handle_transport().await {
            Ok(()) => (),
            Err(e) => return Err(e)
        }

        Ok(())
    }

    async fn handle_handshake(&mut self) -> Result<(), String> {
        // Receive the client hello message
        let n = match self.socket.read(&mut self.buf).await {
            Ok(n) => n,
            Err(e) => {
                warn!("failed to read from socket; err = {:?}", e);
                return Err(String::from("Failed to read socket"));
            }
        };

        debug!("Read {} bytes of data: {:?}", n, &self.buf[0..n]);

        // TODO: Validate client hello message

        // Reply with server hello message
        let server_hello = ServerHello::new(5, 0);
        if let Err(e) = self.socket.write_all(&server_hello.to_bytes()).await {
            error!("failed to write to socket; err = {:?}", e);
        }

        debug!("Wrote {} bytes of data: {:?}", 2, server_hello.to_bytes());

        Ok(())
    }

    async fn handle_request(&mut self) -> Result<Request, String> {
        let n = match self.socket.read(&mut self.buf).await {
            Ok(n) => n,
            Err(e) => {
                warn!("failed to read from socket; err = {:?}", e);
                return Err(String::from("Failed to read socket"));
            }
        };

        debug!("Read {} bytes of data: {:?}", n, &self.buf[0..n]);

        return parser::parse(&self.buf);
    }

    async fn handle_request_ack(&mut self) -> Result<(), String> {
        let ack = RequestAck::new(5, 0, 0, 1, [127, 0, 0, 1], [0x1f, 0x90]);
        if let Err(e) = self.socket.write_all(&ack.to_bytes()).await {
            error!("failed to write to socket; err = {:?}", e);
            return Err(e.to_string());
        }

        debug!("Reply request ACK {:?}", ack.to_bytes());

        Ok(())
    }

    async fn handle_transport(&mut self) -> Result<(), String> {
        let mut stream = match TcpStream::connect(
            self.request.as_ref().unwrap().request_addr_port()).await {
            Ok(s) => s,
            Err(e) => return Err(e.to_string())
        };

        info!("Established TCP connection to {}", self.request.as_ref().unwrap().request_addr_port());

        let (mut target_read, mut target_write) = stream.split();
        let (mut source_read, mut source_write) = self.socket.split();

        match future::join(tokio::io::copy(&mut source_read, &mut target_write),
                           tokio::io::copy(&mut target_read, &mut source_write))
            .await {
            (Err(e), _) | (_, Err(e)) => Err(e.to_string()),
            _ => Ok(()),
        };

        Ok(())
    }
}
