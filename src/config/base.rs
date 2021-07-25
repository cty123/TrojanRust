use serde::{Deserialize, Serialize};

use crate::proxy::base::SupportedProtocols;

#[derive(Serialize, Deserialize, Clone)]
pub struct InboundConfig {
    pub address: String,
    pub port: u16,
    pub protocol: SupportedProtocols,
    pub tls: bool,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OutboundConfig {
    pub address: Option<String>,
    pub port: Option<u16>,
    pub protocol: SupportedProtocols,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub inbound: InboundConfig,
    pub outbound: OutboundConfig,
}
