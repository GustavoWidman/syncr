pub trait Packet {
    fn get_type(&self) -> String;
    fn to_bytes(&self) -> Vec<u8>;
}
