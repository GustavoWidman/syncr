pub mod config;
pub mod packets;
pub mod stream;
pub mod sync;

use std::collections::HashMap;
use std::iter::Map;
use std::sync::Arc;

// re-exports
pub(crate) use config::quick_config;
pub(crate) use packets::packetize;
