macro_rules! packet_buffer_mapper {
    ($($static_packet:ident),* ; $($dynamic_packet:ident),*) => {
        pub fn get_buffer_for_type(packet_type: &[u8; 4], size: &Option<crate::common::packets::types::SizePacket>) -> Result<Vec<u8>, anyhow::Error> {
            match (packet_type, size) {
                // Static packets (ignore size parameter)
                $(
                    ($static_packet::TYPE, _) => $static_packet::make_buffer(),
                )*
                // Dynamic packets (require size parameter)
                $(
                    ($dynamic_packet::TYPE, Some(size)) => <$dynamic_packet as crate::common::packets::DynamicPacket>::make_buffer(&size),
                    ($dynamic_packet::TYPE, None) => panic!("Dynamic packet requires size parameter"),
                )*
                _ => panic!("Unknown packet type: {:?}", packet_type)
            }
        }
    };
}

pub(crate) use packet_buffer_mapper;
