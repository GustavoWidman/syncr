pub mod crypto;
mod packet;
pub mod packets;
pub mod sync;

#[derive(Debug)]
pub enum Packets {
    // Authentication(packets::auth::AuthenticationPacket),
    Salt(packets::salt::SaltPacket),
    Size(packets::size::SizePacket),
    Sanity(packets::sanity::SanityPacket),
    // Data(DataPacket),
}

// exports
pub(crate) use packet::make_packet_buffer;
pub(crate) use packet::packetize;
pub(self) use packet::write_dynsized_packet;
pub(self) use packet::write_packet;
pub(crate) use packet::Packet;
