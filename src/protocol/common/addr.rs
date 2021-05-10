use std::convert::TryInto;

pub trait DestinationAddr {
    fn addr(&self) -> String;
}

pub struct IPv4Addr {
    ip_addr: [u8; 4],
}

pub struct IPv6Addr {
    ip_addr: [u8; 16],
}

pub struct DomainName {
    domain_name: [u8; 256],
}

impl DestinationAddr for IPv4Addr {
    fn addr(&self) -> String {
        return format!(
            "{}.{}.{}.{}",
            self.ip_addr[0], self.ip_addr[1], self.ip_addr[2], self.ip_addr[3]
        );
    }
}

impl DestinationAddr for IPv6Addr {
    fn addr(&self) -> String {
        return format!(
            "{:02x?}{:02x?}:{:02x?}{:02x?}:{:02x?}{:02x?}:{:02x?}{:02x?}:{:02x?}{:02x?}:{:02x?}{:02x?}:{:02x?}{:02x?}:{:02x?}{:02x?}",
            self.ip_addr[0],
            self.ip_addr[1],
            self.ip_addr[2],
            self.ip_addr[3],
            self.ip_addr[4],
            self.ip_addr[5],
            self.ip_addr[6],
            self.ip_addr[7],
            self.ip_addr[8],
            self.ip_addr[9],
            self.ip_addr[10],
            self.ip_addr[11],
            self.ip_addr[12],
            self.ip_addr[13],
            self.ip_addr[14],
            self.ip_addr[15],
        );
    }
}

impl DestinationAddr for DomainName {
    fn addr(&self) -> String {
        return String::from_utf8(Vec::from(self.domain_name)).unwrap();
    }
}

impl IPv4Addr {
    pub fn new(buf: &[u8], ptr: usize) -> String {
        return IPv4Addr {
            ip_addr: buf[ptr..ptr + 4].try_into().expect("Incorrect IPv4 format"),
        }
        .addr();
    }
}

impl IPv6Addr {
    pub fn new(buf: &[u8], ptr: usize) -> String {
        return IPv6Addr {
            ip_addr: buf[ptr..ptr + 16]
                .try_into()
                .expect("Incorrect IPv6 format"),
        }
        .addr();
    }
}
