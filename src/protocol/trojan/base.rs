use bytes::{BufMut, BytesMut};

// use crate::protocol::common::addr::{IPV4_SIZE, IPV6_SIZE, DOMAIN_NAME_SIZE, ATYPE_IPV4, ATYPE_IPV6, ATYPE_DOMAIN_NAME};
// use crate::protocol::common::command::{CONNECT, UDP};

use crate::protocol::common::addr::IpAddress;

const CRLF: u16 = 0x0D0A;

pub struct Request {
    hex: [u8; 56],
    command: u8,
    atype: u8,
    addr: IpAddress,
    addr_len: usize,
    port: u16,
}

// pub struct UdpRequest {
//     atype: u8,
//     addr: IpAddr,
//     addr_len: usize,
//     port: u16,
//     payload: [u8; 2048],
//     payload_size: usize,
// }

impl Request {
    pub fn new(
        hex: [u8; 56],
        command: u8,
        atype: u8,
        addr: IpAddress,
        addr_len: usize,
        port: u16,
    ) -> Request {
        return Request {
            hex,
            command,
            atype,
            addr,
            addr_len,
            port,
        };
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(128);
        buf.put_slice(&self.hex);
        buf.put_u16(CRLF);
        buf.put_u8(self.command);
        buf.put_u8(self.atype);
        buf.put_slice(&self.addr.to_bytes());
        buf.put_u16(self.port);
        buf.put_u16(CRLF);
        return buf.to_vec();
    }

    #[inline]
    pub fn request_addr_port(&self) -> String {
        return format!("{}:{}", self.addr.to_string(), self.port);
    }

    #[inline]
    pub fn dump_request(&self) -> String {
        let command = match self.command {
            1 => "Connect",
            2 => "Bind",
            3 => "UDP Associate",
            _ => "Unsupported",
        };
        return format!("[{} => {}]", command, self.request_addr_port());
    }
}

//     pub fn get_command(&self) -> u8 {
//         return self.command;
//     }
// }

// impl UdpRequest {
//     pub fn new(atype: u8, addr: [u8; 256], addr_len: usize, port: u16, payload: [u8; 2048],
//                payload_size: usize) -> UdpRequest {
//         UdpRequest {
//             atype,
//             addr,
//             addr_len,
//             port,
//             payload,
//             payload_size
//         }
//     }

//     pub fn request_addr_port(&self) -> String {
//         let addr = match self.atype {
//             ATYPE_IPV4 => ipv4_to_string(self.addr[0..IPV4_SIZE].try_into().unwrap()),
//             ATYPE_IPV6 => ipv6_to_string(self.addr[0..IPV6_SIZE].try_into().unwrap()),
//             ATYPE_DOMAIN_NAME => String::from_utf8_lossy(&self.addr[0..self.addr_len]).to_string(),
//             _ => String::from("Unknown")
//         };
//         return format!("{}:{}", addr, self.port);
//     }

//     pub fn dump_request(&self) -> String {
//         let atype = match self.atype {
//             ATYPE_IPV4 => "IPv4",
//             ATYPE_IPV6 => "IPv6",
//             ATYPE_DOMAIN_NAME => "DomainName",
//             _ => "Unsupported"
//         };

//         return format!(
//             "[{} {}::{}]",
//             self.payload_size,
//             atype,
//             self.request_addr_port(),
//         );
//     }
// }
