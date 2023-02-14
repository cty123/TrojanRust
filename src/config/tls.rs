use log::error;

use std::fs::File;
use std::io::{BufReader, ErrorKind};
use std::sync::Arc;
use std::time::SystemTime;

use rustls::client::{ServerCertVerified, ServerCertVerifier, ServerName};
use rustls::Error;
use rustls::RootCertStore;
use rustls::{Certificate, ClientConfig, PrivateKey, ServerConfig};
use rustls_pemfile::{read_one, Item};

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
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: SystemTime,
    ) -> Result<ServerCertVerified, Error> {
        Ok(ServerCertVerified::assertion())
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
pub fn make_client_config(config: &OutboundTlsConfig) -> ClientConfig {
    if config.allow_insecure {
        let mut config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(RootCertStore::empty())
            .with_no_client_auth();

        config
            .dangerous()
            .set_certificate_verifier(Arc::new(NoCertificateVerification {}));

        config
    } else {
        let mut root_store = RootCertStore::empty();
        root_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
            rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
                ta.subject,
                ta.spki,
                ta.name_constraints,
            )
        }));

        let config = ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        config
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
pub fn make_server_config(config: &InboundTlsConfig) -> Option<ServerConfig> {
    let certificates = match load_certs(&config.cert_path) {
        Ok(certs) => certs,
        Err(_) => return None,
    };

    let key = match load_private_key(&config.key_path) {
        Ok(key) => key,
        Err(_) => return None,
    };

    let cfg = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certificates, key)
        .expect("bad certificate/key");

    Some(cfg)
}

fn load_certs(path: &str) -> std::io::Result<Vec<Certificate>> {
    let mut reader = match File::open(path) {
        Ok(file) => BufReader::new(file),
        Err(e) => {
            error!("Failed to load tls certificate file, {}", e);
            return Err(e);
        }
    };

    return match rustls_pemfile::certs(&mut reader) {
        Ok(certs) => Ok(certs.into_iter().map(|bytes| Certificate(bytes)).collect()),
        Err(_) => Err(std::io::Error::new(
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
                Item::RSAKey(key) => Ok(rustls::PrivateKey(key)),
                Item::PKCS8Key(key) => Ok(rustls::PrivateKey(key)),
                Item::ECKey(key) => Ok(rustls::PrivateKey(key)),
                _ => Err(std::io::Error::new(
                    ErrorKind::InvalidInput,
                    "Found cert in ssl key file",
                )),
            },
            None => Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                "Failed to find any private key in file",
            )),
        },
        Err(e) => Err(e),
    };
}
