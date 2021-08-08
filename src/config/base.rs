use serde::{Deserialize, Serialize};

use crate::proxy::base::SupportedProtocols;

#[derive(Serialize, Deserialize, Clone)]
pub struct InboundConfig {
    pub protocol: SupportedProtocols,
    pub address: String,
    pub port: u16,

    pub tls: Option<TlsConfig>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OutboundConfig {
    pub protocol: SupportedProtocols,
    pub address: Option<String>,
    pub port: Option<u16>,

    pub tls: Option<TlsConfig>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TlsConfig {
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
    pub host_name: Option<String>,
    pub allow_insecure: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub inbound: InboundConfig,
    pub outbound: OutboundConfig,
}
