use clap::ArgMatches;
use clap::{App, Arg};
use lazy_static::lazy_static;
use std::io::Result;
use trojan_rust::config::base::{Config, InboundConfig, OutboundConfig};
use trojan_rust::config::parser::reader_config;
use trojan_rust::proxy::tcp;
use trojan_rust::proxy::tcp::acceptor::Acceptor;
use trojan_rust::proxy::tcp::handler::Handler;
// use trojan_rust::proxy::grpc_server::GrpcServer;

lazy_static! {
    static ref ARGS: ArgMatches = {
        App::new("Trojan Rust")
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
            .get_matches()
    };
    static ref CONFIG_PATH: &'static str =
        ARGS.value_of("config").unwrap_or("./config/config.json");
    static ref CONFIG: Config = reader_config(&CONFIG_PATH).unwrap();
    static ref INBOUND_CONFIG: InboundConfig = CONFIG.inbound.clone();
    static ref OUTBOUND_CONFIG: OutboundConfig = CONFIG.outbound.clone();
    static ref ACCEPTOR: Acceptor = Acceptor::new(&INBOUND_CONFIG);
    static ref HANDLER: Handler = Handler::new(&OUTBOUND_CONFIG).unwrap();
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let (local_addr, port) = (INBOUND_CONFIG.address.clone(), INBOUND_CONFIG.port);

    return match INBOUND_CONFIG.transport {
        // Some(_protocol) if matches!(TransportProtocol::GRPC, _protocol) => {
        //     start_grpc_server(config.inbound, config.outbound).await
        // }
        _ => {
            tcp::server::start(
                format!("{}:{}", local_addr, port).parse().unwrap(),
                &ACCEPTOR,
                &HANDLER,
            )
            .await
        }
    };
}

// async fn start_grpc_server(
//     inbound_config: InboundConfig,
//     outbound_config: OutboundConfig,
// ) -> Result<()> {
//     let server = match GrpcServer::new(inbound_config, outbound_config).await {
//         Ok(server) => server,
//         Err(e) => {
//             error!("Failed to instantiate the server, {}", e);
//             return Err(e);
//         }
//     };

//     return match server.start().await {
//         Err(e) => {
//             error!("Server failure: {}, graceful shutdown", e.to_string());
//             Err(e)
//         }
//         Ok(()) => {
//             info!("Finished running server, exiting...");
//             Ok(())
//         }
//     };
// }
