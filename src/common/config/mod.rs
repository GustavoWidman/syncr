mod primary;
pub mod sync;

pub use primary::store::Config;
pub(crate) use primary::store::quick_config;
pub use primary::structure::{self, ConfigTOML};

pub use sync::store::SyncConfig;
pub use sync::structure::SyncConfigTOML;
