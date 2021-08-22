use std::fmt::{self};
use std::io::{Error, ErrorKind, Result};

const CONNECT: u8 = 1;
const BIND: u8 = 2;
const UDP: u8 = 3;

#[derive(Copy, Clone)]
pub enum Command {
    Connect,
    Bind,
    Udp,
}

impl Command {
    #[inline]
    pub fn from(command: u8) -> Result<Command> {
        return match command {
            CONNECT => Ok(Command::Connect),
            BIND => Ok(Command::Bind),
            UDP => Ok(Command::Udp),
            _ => Err(Error::new(
                ErrorKind::Unsupported,
                "Unsupported command request",
            )),
        };
    }

    #[inline]
    pub fn to_byte(&self) -> u8 {
        match self {
            Command::Connect => CONNECT,
            Command::Bind => BIND,
            Command::Udp => UDP,
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Connect => write!(fmt, "Connect"),
            Command::Bind => write!(fmt, "Bind"),
            Command::Udp => write!(fmt, "Udp"),
        }
    }
}
