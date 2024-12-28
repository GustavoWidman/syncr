use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use diesel::prelude::*;
use diesel_migrations::{EmbeddedMigrations, embed_migrations};

use crate::data::DatabaseDriver;

pub const COMMON_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/common");
pub const SERVER_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/server");

pub struct ServerDatabase {
    inner: SqliteConnection,
}

impl DatabaseDriver for ServerDatabase {
    const MIGRATIONS: [EmbeddedMigrations; 2] = [COMMON_MIGRATIONS, SERVER_MIGRATIONS];

    async fn new(path: Option<PathBuf>) -> Result<Self, anyhow::Error> {
        let mut database = Self::connect(path).await.map_err(|e| anyhow::anyhow!(e))?;

        Self::migrate(&mut database).await?;

        Ok(Self { inner: database })
    }
}

impl Deref for ServerDatabase {
    type Target = SqliteConnection;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ServerDatabase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
