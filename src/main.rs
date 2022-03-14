use clap::ArgMatches;
use clap::{App, Arg};
use lazy_static::lazy_static;
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

    let config_path = ARGS.value_of("config").unwrap_or("./config/config.json");
    let config = reader_config(config_path).unwrap();
    let (inbound_config, outbound_config) = (config.inbound, config.outbound);

    match inbound_config.transport {
        Some(_protocol) if matches!(TransportProtocol::GRPC, _protocol) => {
            grpc::server::start(inbound_config, outbound_config).await;
        }
        _ => {
            tcp::server::start(inbound_config, outbound_config).await;
        }
    };

    Ok(())
}
