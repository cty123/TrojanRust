use super::base::Request;
use log::{debug, error, info, warn};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub struct Handler {
    socket: TcpStream,
    request: Option<Request>,
    buf: [u8; 1024],
}

impl Handler {
    pub fn new(mut socket: TcpStream) -> Handler {
        Handler {
            socket: socket,
            request: None,
            buf: [0; 1024],
        }
    }

    pub async fn handle(&mut self) -> Result<(), String> {
        match self.handle_handshake().await {
            Ok(_) => return Ok(()),
            Err(e) => return Err(e)
        };
    }

    async fn handle_handshake(&mut self) -> Result<(), String> {
        let n = match self.socket.read(&mut self.buf).await {
            Ok(n) => n,
            Err(e) => {
                warn!("failed to read from socket; err = {:?}", e);
                return Err(String::from("Failed to read socket"));
            }
        };


        Ok(())
    }

    fn handle_request_ack(&self) {}

    fn handle_request(&self) {}
}
