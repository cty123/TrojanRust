use log::{error, info};

use std::io::{Error, ErrorKind, Result};

use clap::{App, Arg, SubCommand};

// use crate::proxy::base::SupportedProtocols;

mod config;
mod infra;
mod protocol;
mod proxy;
mod transport;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let matches = App::new("Rust Proxy")
        .version("1.0.0")
        .author("Anonymous")
        .about("Rust proxy")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true),
        )
        .get_matches();

    let config_path = matches.value_of("config").unwrap_or("./config.json");

    info!("Parsing Rust-proxy configuration from {}", config_path);

    let config = match config::parser::reader_config(config_path) {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load config file, {}", e);
            return Err(Error::new(ErrorKind::InvalidInput, e));
        }
    };

    // let mut cfg = rustls::ServerConfig::new(rustls::NoClientAuth::new());
    // info!("Starting Rust-proxy at {}", "127.0.0.1:8080");

    let server = proxy::tcp_server::TcpServer::new(config.inbound, config.outbound);

    // match server.start().await {
    //     Err(e) => info!("Server failure: {}", e.to_string()),
    //     Ok(()) => info!("Finished running server, exiting..."),
    // }

    Ok(())
}

// fn setup_certificate(cert_path: &str, key_path: &str) -> Result<ServerConfig, String> {
//     let certs = load_certs(cert_path).unwrap();
//     let mut keys = load_keys(key_path).unwrap();

//     let mut config = ServerConfig::new(NoClientAuth::new());
//     config
//         .set_single_cert(certs, keys.remove(0))
//         .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))
//         .unwrap();

//     Ok(config)
// }

// fn load_certs(path: &str) -> io::Result<Vec<Certificate>> {
//     certs(&mut BufReader::new(File::open(path)?))
//         .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid cert"))
// }

// fn load_keys(path: &str) -> io::Result<Vec<PrivateKey>> {
//     pkcs8_private_keys(&mut BufReader::new(File::open(path)?))
//         .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid key"))
// }
