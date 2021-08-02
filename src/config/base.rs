use rustls::{Certificate, RootCertStore, ServerCertVerified, ServerCertVerifier, TLSError};
use serde::{Deserialize, Serialize};
use tokio_rustls::webpki::DNSNameRef;

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
    pub tls: bool,
    pub server_name: Option<String>,
    pub insecure: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub inbound: InboundConfig,
    pub outbound: OutboundConfig,
}

pub struct NoCertificateVerification {}

impl ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _roots: &RootCertStore,
        _presented_certs: &[Certificate],
        _dns_name: DNSNameRef,
        _ocsp_response: &[u8],
    ) -> Result<ServerCertVerified, TLSError> {
        Ok(rustls::ServerCertVerified::assertion())
    }
}
