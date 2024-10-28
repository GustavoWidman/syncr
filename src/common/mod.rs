mod packet;
pub mod packets;

pub enum Packets {
    Authentication(packets::auth::AuthenticationPacket),
    // Data(DataPacket),
}

// exports
pub(crate) use packet::Packet;
