use log::{error, warn};

use std::fs::File;
use std::io::{BufReader, Error, ErrorKind, Result};
use std::sync::Arc;

use rustls::internal::pemfile;
use rustls::{Certificate, NoClientAuth, PrivateKey, ServerConfig};

pub fn get_tls_config(cert_path: &str, key_path: &str) -> Result<Arc<ServerConfig>> {
    let certificates = load_certs(cert_path)?;
    let key = load_private_key(key_path)?;

    let mut cfg = rustls::ServerConfig::new(NoClientAuth::new());
    cfg.set_single_cert(certificates, key);

    Ok(Arc::new(cfg))
}

fn load_certs(path: &str) -> Result<Vec<Certificate>> {
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

fn load_private_key(path: &str) -> Result<PrivateKey> {
    let mut reader = match File::open(path) {
        Ok(file) => BufReader::new(file),
        Err(e) => {
            error!("Failed to load tls certificate file, {}", e);
            return Err(e);
        }
    };

    return match pemfile::pkcs8_private_keys(&mut reader) {
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
