use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use diesel::prelude::*;
use diesel_migrations::{EmbeddedMigrations, embed_migrations};

use crate::data::DatabaseDriver;

pub const COMMON_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/common");
pub const CLIENT_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/client");

pub struct ClientDatabase {
    inner: SqliteConnection,
}

impl DatabaseDriver for ClientDatabase {
    const MIGRATIONS: [EmbeddedMigrations; 2] = [COMMON_MIGRATIONS, CLIENT_MIGRATIONS];

    async fn new(path: Option<PathBuf>) -> Result<Self, anyhow::Error> {
        let mut database = Self::connect(path).await.map_err(|e| anyhow::anyhow!(e))?;

        Self::migrate(&mut database).await?;

        Ok(Self { inner: database })
    }
}

impl Deref for ClientDatabase {
    type Target = SqliteConnection;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ClientDatabase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
