use log::{error, info};

use std::io::{Error, ErrorKind, Result};

use clap::{App, Arg};

mod config;
mod infra;
mod protocol;
mod proxy;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let matches = App::new("Trojan Rust")
        .version("0.1.0")
        .author("cty123")
        .about("Trojan Rust is a rust implementation of the trojan protocol to circumvent GFW")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets the config file, read ./config/config.json by default")
                .takes_value(true),
        )
        .get_matches();

    let config_path = matches.value_of("config").unwrap_or("./config/config.json");

    info!("Parsing trojan-rust configuration from {}", config_path);

    let config = match config::parser::reader_config(config_path) {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load config file, {}", e);
            return Err(Error::new(ErrorKind::InvalidInput, e));
        }
    };

    let server = match proxy::tcp_server::TcpServer::new(config.inbound, config.outbound) {
        Ok(server) => server,
        Err(e) => {
            error!("Failed to instantiate the server, {}", e);
            return Err(e);
        }
    };

    match server.start().await {
        Err(e) => info!("Server failure: {}", e.to_string()),
        Ok(()) => info!("Finished running server, exiting..."),
    }

    Ok(())
}
