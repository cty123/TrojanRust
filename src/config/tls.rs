use log::{error, warn};

use std::fs::File;
use std::io::{BufReader, Error, ErrorKind, Result};

use rustls::internal::pemfile;
use rustls::{Certificate, PrivateKey};

pub fn load_certs(path: &str) -> Result<Vec<Certificate>> {
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

pub fn load_private_key(path: &str) -> Result<PrivateKey> {
    let mut reader = match File::open(path) {
        Ok(file) => BufReader::new(file),
        Err(e) => {
            error!("Failed to load tls certificate file, {}", e);
            return Err(e);
        }
    };

    return match pemfile::rsa_private_keys(&mut reader) {
        Ok(keys) if keys.len() == 1 => Ok(keys.first().unwrap().clone()),
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
