use crate::proxy::base::SupportedProtocols;

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub inbound: InboundConfig,
    pub outbound: OutboundConfig,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum InboundMode {
    TCP,
    GRPC,
    QUIC,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum OutboundMode {
    DIRECT,
    GRPC,
    QUIC,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct InboundConfig {
    pub mode: InboundMode,
    pub protocol: SupportedProtocols,
    pub address: String,
    pub port: u16,
    pub secret: Option<String>,
    pub tls: Option<InboundTlsConfig>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OutboundConfig {
    pub mode: OutboundMode,
    pub protocol: SupportedProtocols,
    pub address: Option<String>,
    pub port: Option<u16>,
    pub secret: Option<String>,
    pub tls: Option<OutboundTlsConfig>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct InboundTlsConfig {
    pub cert_path: String,
    pub key_path: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OutboundTlsConfig {
    pub host_name: String,
    pub allow_insecure: bool,
}

impl fmt::Display for InboundMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for OutboundMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}
