use crate::proxy::base::SupportedProtocols;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub inbound: InboundConfig,
    pub outbound: OutboundConfig,
}

/// Inbound traffic supports the following 3 modes: 
/// 
/// TCP - Raw TCP byte stream traffic
/// GRPC - GRPC packet stream that contains payload data in the body for proxy purposes
/// QUIC - Application level byte stream that is built on top of QUIC protocol
/// 
/// TCP and QUIC are both byte streams from the abstractions of the low level implementation. GRPC on the other hand is 
/// packet stream.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum InboundMode {
    TCP,
    GRPC,
    QUIC,
}

/// Outbound traffic supports 4 types of proxy modes:
/// 
/// DIRECT: Directly send the data in the proxy request to the requested destination, either via raw TCP or UDP
/// TCP: Forward the proxy traffic to a remote proxy server via raw TCP stream and have it take care of the traffic handling
/// GRPC: Forward the proxy traffic to a remote proxy server via GRPC packet stream
/// QUIC: Forward the proxy traffic to a remote proxy server via QUIC stream
#[derive(Serialize, Deserialize, Clone)]
pub enum OutboundMode {
    DIRECT,
    TCP,
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
