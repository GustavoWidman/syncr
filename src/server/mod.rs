mod config;
pub mod database; // todo remove pub
pub mod handlers;
mod init;

pub(crate) use init::Server;
