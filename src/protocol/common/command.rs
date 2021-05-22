use std::fmt::{Display, Formatter, Result};

pub enum Command {
    CONNECT = 1,
    BIND = 2,
    UDPASSOCIATE = 3,
}

impl Display for Command {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Command::CONNECT => write!(f, "CONNECT"),
            Command::BIND => write!(f, "BIND"),
            Command::UDPASSOCIATE => write!(f, "UDPASSOCIATE"),
        }
    }
}
