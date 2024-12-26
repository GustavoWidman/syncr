mod primary;
mod sync;

pub use primary::store::Config;
pub(crate) use primary::store::quick_config;
pub use primary::structure::{self, ConfigTOML};
