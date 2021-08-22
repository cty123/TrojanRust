use std::fs::File;
use std::io::{BufReader, Error, ErrorKind, Result};

use crate::config::base::Config;

/// Read and parse the json file located at path, will attempt to deserialize and throw error if the
/// format is invalid.
pub fn reader_config(path: &str) -> Result<Config> {
    let reader = match File::open(path) {
        Ok(file) => BufReader::new(file),
        Err(e) => return Err(Error::new(ErrorKind::InvalidData, e)),
    };

    return match serde_json::from_reader(reader) {
        Ok(config) => Ok(config),
        Err(e) => return Err(Error::new(ErrorKind::InvalidData, e)),
    };
}
