use log::{error};

use std::fs::File;
use std::io::{BufReader, Error, ErrorKind};
use std::sync::Arc;

use rustls::internal::pemfile;
use rustls::{
    Certificate, ClientConfig, NoClientAuth, PrivateKey, RootCertStore, ServerCertVerified,
    ServerCertVerifier, ServerConfig, TLSError,
};
use rustls_pemfile::{read_one, Item};
use tokio_rustls::webpki::DNSNameRef;

use crate::config::base::{InboundTlsConfig, OutboundTlsConfig};

/// Stub Certificate verifier that skips certificate verification. It is used when the user
/// explicitly allows insecure TLS connection in configuration file, by setting
///
/// ```json
/// {
///     ...,
///     outbound: {
///         ...,
///         tls: {
///             ...,
///             allow_insecure: true
///         }
///     }
/// }
/// ```
///
/// The option is not recommended for production level services, but could be handy in testing stages.
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

/// Create ClientConfig for rustls based on the configurations in the config.json file. The function
/// will read the tls configuration under outbound,
///
/// ```json
/// {
///     outbound: {
///         tls: {
///             # Configurations here
///         }
///     }         
/// }
/// ```
pub fn make_client_config(config: &OutboundTlsConfig) -> Arc<ClientConfig> {
    if config.allow_insecure {
        let mut config = ClientConfig::default();
        config
            .dangerous()
            .set_certificate_verifier(Arc::new(NoCertificateVerification {}));
        Arc::new(config)
    } else {
        let mut config = ClientConfig::default();
        config
            .root_store
            .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
        Arc::new(config)
    }
}

/// Create ServerConfig for rustls based on the configurations in the config.json file. The function
/// will read the tls configuration under inbound,
///
/// ```json
/// {
///     inbound: {
///         tls: {
///             # Configurations here
///         }
///     }         
/// }
/// ```
pub fn make_server_config(config: &InboundTlsConfig) -> Option<Arc<ServerConfig>> {
    let certificates = match load_certs(&config.cert_path) {
        Ok(certs) => certs,
        Err(_) => return None,
    };

    let key = match load_private_key(&config.key_path) {
        Ok(key) => key,
        Err(_) => return None,
    };

    let mut cfg = rustls::ServerConfig::new(NoClientAuth::new());

    match cfg.set_single_cert(certificates, key) {
        Ok(_) => Some(Arc::new(cfg)),
        Err(_) => None,
    }
}

fn load_certs(path: &str) -> std::io::Result<Vec<Certificate>> {
    let mut reader = match File::open(path) {
        Ok(file) => BufReader::new(file),
        Err(e) => {
            error!("Failed to load tls certificate file, {}", e);
            return Err(e);
        }
    };

    return match pemfile::certs(&mut reader) {
        Ok(certs) => Ok(certs),
        Err(_) => Err(Error::new(
            ErrorKind::InvalidData,
            "failed to load tls certificate",
        )),
    };
}

fn load_private_key(path: &str) -> std::io::Result<PrivateKey> {
    let mut reader = match File::open(path) {
        Ok(file) => BufReader::new(file),
        Err(e) => return Err(e),
    };

    return match read_one(&mut reader) {
        Ok(opt) => match opt {
            Some(item) => match item {
                Item::X509Certificate(_) => Err(Error::new(
                    ErrorKind::InvalidInput,
                    "Found cert in ssl key file",
                )),
                Item::RSAKey(key) => Ok(rustls::PrivateKey(key)),
                Item::PKCS8Key(key) => Ok(rustls::PrivateKey(key)),
            },
            None => Err(Error::new(
                ErrorKind::InvalidInput,
                "Failed to find any private key in file",
            )),
        },
        Err(e) => Err(e),
    };
}
