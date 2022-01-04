use std::io::{Error, ErrorKind, Result};

use clap::{App, Arg};
use log::{error, info};

use trojan_rust::config::base::{InboundConfig, OutboundConfig};
use trojan_rust::config::parser::reader_config;
use trojan_rust::protocol::common::request::TransportProtocol;
use trojan_rust::proxy::grpc_server::GrpcServer;
use trojan_rust::proxy::tcp_server::TcpServer;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let matches = App::new("Trojan Rust")
        .version("0.4")
        .author("cty123")
        .about("Trojan Rust is a rust implementation of the trojan protocol to circumvent GFW")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Sets the config file, read ./config/config.json by default")
                .takes_value(true),
        )
        .get_matches();

    let config_path = matches.value_of("config").unwrap_or("./config/config.json");

    info!("Parsing trojan-rust configuration from {}", config_path);

    let config = match reader_config(config_path) {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load config file, {}", e);
            return Err(Error::new(ErrorKind::InvalidInput, e));
        }
    };

    return match config.inbound.transport {
        Some(_protocol) if matches!(TransportProtocol::GRPC, _protocol) => {
            start_grpc_server(config.inbound, config.outbound).await
        }
        _ => start_tcp_server(config.inbound, config.outbound).await,
    };
}

async fn start_tcp_server(
    inbound_config: InboundConfig,
    outbound_config: OutboundConfig,
) -> Result<()> {
    let server = match TcpServer::new(inbound_config, outbound_config) {
        Ok(server) => server,
        Err(e) => {
            error!("Failed to instantiate the server, {}", e);
            return Err(e);
        }
    };

    match server.start().await {
        Err(e) => info!("Server failure: {}, graceful shutdown", e.to_string()),
        Ok(()) => info!("Finished running server, exiting..."),
    }

    Ok(())
}

async fn start_grpc_server(
    inbound_config: InboundConfig,
    outbound_config: OutboundConfig,
) -> Result<()> {
    let server = match GrpcServer::new(inbound_config, outbound_config) {
        Ok(server) => server,
        Err(e) => {
            error!("Failed to instantiate the server, {}", e);
            return Err(e);
        }
    };

    return match server.start().await {
        Err(e) => {
            error!("Server failure: {}, graceful shutdown", e.to_string());
            Err(e)
        }
        Ok(()) => {
            info!("Finished running server, exiting...");
            Ok(())
        }
    };
}
