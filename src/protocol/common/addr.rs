pub const IPV4_SIZE: usize = 4;
pub const DOMAIN_NAME_SIZE: usize = 256;
pub const IPV6_SIZE: usize = 16;

pub const ATYPE_IPV4: u8 = 1;
pub const ATYPE_DOMAIN_NAME: u8 = 3;
pub const ATYPE_IPV6: u8 = 4;

pub fn ipv4_to_string(addr: [u8; 4]) -> String {
    return format!(
        "{}.{}.{}.{}",
        addr[0], addr[1], addr[2], addr[3]
    );
}

pub fn ipv6_to_string(addr: [u8; 16]) -> String {
    return format!(
        "{:02x?}{:02x?}:{:02x?}{:02x?}:{:02x?}{:02x?}:{:02x?}{:02x?}:{:02x?}{:02x?}:{:02x?}{:02x?}:{:02x?}{:02x?}:{:02x?}{:02x?}",
        addr[0],
        addr[1],
        addr[2],
        addr[3],
        addr[4],
        addr[5],
        addr[6],
        addr[7],
        addr[8],
        addr[9],
        addr[10],
        addr[11],
        addr[12],
        addr[13],
        addr[14],
        addr[15],
    );
}

pub fn domain_name_to_string(addr: [u8; 256]) -> String {
    return String::from_utf8(addr.to_vec()).unwrap();
}