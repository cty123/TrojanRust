use clap::ArgMatches;
use clap::{App, Arg};
use lazy_static::lazy_static;
use log::info;
use std::io::Result;
use trojan_rust::config::parser::reader_config;
use trojan_rust::protocol::common::request::TransportProtocol;
use trojan_rust::proxy::grpc;
use trojan_rust::proxy::tcp;

lazy_static! {
    static ref ARGS: ArgMatches = App::new("Trojan Rust")
        .version("0.5")
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
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // Read configuration file from arguments
    let config_path = ARGS.value_of("config").unwrap_or("./config/config.json");
    info!("Reading trojan configuration file from: {}", config_path);

    // Error out immediately if failing to parse config file
    let config = reader_config(config_path).unwrap();

    // Extract inbound and outbound configuration
    let (inbound_config, outbound_config) = (config.inbound, config.outbound);

    // Currently only supporting GRPC server and TCP server
    // TODO: Add a dedicated field in order to indicate which type of server to use
    // TODO: Support more types of server, like UDP
    match inbound_config.transport {
        Some(_protocol) if matches!(TransportProtocol::GRPC, _protocol) => {
            info!("Starting GRPC server to accept inbound traffic");
            grpc::server::start(inbound_config, outbound_config).await?;
        }
        _ => {
            info!("Starting Tcp server to accept inbound traffic");
            tcp::server::start(inbound_config, outbound_config).await?;
        }
    };

    Ok(())
}
