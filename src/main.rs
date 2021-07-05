use log::{info, warn};
use tokio::net::{TcpListener, UdpSocket};
use rustls::internal::pemfile::{certs, pkcs8_private_keys};
use std::io::BufReader;
use std::fs::File;
use std::io;
use rustls::{NoClientAuth, ServerConfig, Certificate, PrivateKey};
use tokio_rustls::TlsAcceptor;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};

mod transport;
mod protocol;
mod config;
mod application;
mod infra;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Initialize configurations
    env_logger::init();
    info!("Starting Rust-proxy at {}", "127.0.0.1:8080");

    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    // TLS
    let config = setup_certificate("./cert/test.crt", "./cert/test.key").unwrap();
    let acceptor = TlsAcceptor::from(Arc::new(config));

    loop {
        let (mut socket, _) = listener.accept().await?;
        let acceptor = acceptor.clone();

        tokio::spawn(async move {
            if true {
                let stream = match acceptor.accept(socket).await {
                    Ok(stream) => stream,
                    Err(_) => return
                };
                dispatch(stream).await;
            } else {
                dispatch(socket).await;
            }
        });
    }
}

async fn dispatch<IO>(socket: IO)
    where IO: AsyncRead + AsyncWrite + Unpin
{
    match transport::tcp::dispatch(socket, "server").await {
        Ok(_) => {
            info!("Finished processing socket");
        }
        Err(e) => {
            warn!("Error in dispatching the TCP socket: {}", e);
        }
    }
}

fn setup_certificate(cert_path: &str, key_path: &str) -> Result<ServerConfig, String> {
    let certs = load_certs(cert_path).unwrap();
    let mut keys = load_keys(key_path).unwrap();

    let mut config = ServerConfig::new(NoClientAuth::new());
    config.set_single_cert(certs, keys.remove(0))
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))
        .unwrap();

    Ok(config)
}

fn load_certs(path: &str) -> io::Result<Vec<Certificate>> {
    certs(&mut BufReader::new(File::open(path)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid cert"))
}

fn load_keys(path: &str) -> io::Result<Vec<PrivateKey>> {
    pkcs8_private_keys(&mut BufReader::new(File::open(path)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid key"))
}
