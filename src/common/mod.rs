pub mod crypto;
mod packet;
pub mod packets;
pub mod sync;

pub enum Packets {
    // Authentication(packets::auth::AuthenticationPacket),
    Salt(packets::salt::SaltPacket),
    // Data(DataPacket),
}

// exports
pub(crate) use packet::find_packet_type;
pub(crate) use packet::Packet;
