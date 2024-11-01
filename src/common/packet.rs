use std::io;

use super::packets::auth::AuthenticationPacket;

pub trait Packet {
    type BuildParams;
    fn build(params: Self::BuildParams) -> Self;
    fn get_type(&self) -> &[u8; 4];
    fn from_bytes(bytes: &[u8]) -> Self;
    fn to_bytes(&self) -> Vec<u8>;
}

pub fn find_packet_type(packet: &[u8]) -> io::Result<(&str, &[u8])> {
    if packet.len() < 4 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Packet too small",
        ));
    }

    let packet_type = std::str::from_utf8(&packet[0..4])
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let packet_bytes = &packet[4..];

    Ok((packet_type, packet_bytes))
}
