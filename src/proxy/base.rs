use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum SupportedProtocols {
    DIRECT,
    SOCKS,
    TROJAN,
}
