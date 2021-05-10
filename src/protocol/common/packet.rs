pub trait Packet {
    fn to_bytes(&self) -> &[u8];
}