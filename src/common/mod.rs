pub mod noise;
mod packet;
pub mod packets;

pub enum Packets {
    Authentication(packets::auth::AuthenticationPacket),
    // Data(DataPacket),
}

// exports
pub(crate) use packet::find_packet_type;
pub(crate) use packet::Packet;
