use log::{error, warn};

use std::fs::File;
use std::io::{BufReader, Error, ErrorKind};
use std::sync::Arc;

use rustls::internal::pemfile;
use rustls::{
    Certificate, ClientConfig, NoClientAuth, PrivateKey, RootCertStore, ServerCertVerified,
    ServerCertVerifier, ServerConfig, TLSError,
};
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
        Err(e) => {
            error!("Failed to load tls certificate file, {}", e);
            return Err(e);
        }
    };

    // Try to load pkcs8 private key in key file 
    match load_pkcs8_private_key(&mut reader, path) {
        Ok(key) => return Ok(key),
        Err(_) => (),
    };

    // Try to load rsa private key in key file
    match load_rsa_private_key(&mut reader, path) {
        Ok(key) => return Ok(key),
        Err(_) => (),
    };

    Err(Error::new(ErrorKind::InvalidInput, "Failed to find any private key in file"))
}

fn load_pkcs8_private_key(
    reader: &mut BufReader<std::fs::File>,
    path: &str,
) -> std::io::Result<PrivateKey> {
    return match pemfile::pkcs8_private_keys(reader) {
        Ok(keys) if keys.len() == 1 => Ok(keys.first().unwrap().clone()),
        Ok(keys) if keys.len() < 1 => {
            error!("No private key found in file {}", path);
            Err(Error::new(ErrorKind::InvalidData, "no private key found"))
        }
        Ok(keys) => {
            warn!(
                "Multiple private keys found in file {}, will take the first one",
                path
            );
            Ok(keys.first().unwrap().clone())
        }
        Err(_) => Err(Error::new(
            ErrorKind::InvalidData,
            "failed to load PKCS8 private key",
        )),
    };
}

fn load_rsa_private_key(
    reader: &mut BufReader<std::fs::File>,
    path: &str,
) -> std::io::Result<PrivateKey> {
    return match pemfile::rsa_private_keys(reader) {
        Ok(keys) if keys.len() == 1 => Ok(keys.first().unwrap().clone()),
        Ok(keys) if keys.len() < 1 => {
            error!("No private key found in file {}", path);
            Err(Error::new(ErrorKind::InvalidData, "no private key found"))
        }
        Ok(keys) => {
            warn!(
                "Multiple private keys found in file {}, will take the first one",
                path
            );
            Ok(keys.first().unwrap().clone())
        }
        Err(_) => Err(Error::new(
            ErrorKind::InvalidData,
            "failed to load RSA private key",
        )),
    };
}
