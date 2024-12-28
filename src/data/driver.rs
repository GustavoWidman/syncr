use std::{
    fs,
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use diesel::{Connection, SqliteConnection};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use dirs::home_dir;

pub trait DatabaseDriver: Deref<Target = SqliteConnection> + DerefMut + Sized {
    const MIGRATIONS: [EmbeddedMigrations; 2];

    async fn connect(path: Option<PathBuf>) -> Result<SqliteConnection, anyhow::Error> {
        // absolutely beautiful...

        // if this path exists yet its not a file replace with None, which ends up turning default
        path.and_then(|p| (p.is_file() || !p.exists()).then_some(p))
            .or_else(|| home_dir().map(|dir| dir.join(".syncr").join("syncr.db"))) // default
            .ok_or(anyhow::anyhow!(
                "Unable to extract config path, default home directory not found."
            ))
            .and_then(|path| {
                if !path.exists() {
                    fs::create_dir_all(
                        path.parent()
                            .ok_or(anyhow::anyhow!("Unable to get parent dir"))?,
                    )?;
                    fs::File::create(&path)?;
                }
                Ok(path)
            })?
            .canonicalize()?
            .to_str()
            .ok_or(anyhow::anyhow!("Unable to convert path to string"))
            .and_then(|path_str| SqliteConnection::establish(&path_str).map_err(Into::into))
    }

    async fn new(path: Option<PathBuf>) -> Result<Self, anyhow::Error>;

    async fn migrate(database: &mut SqliteConnection) -> Result<(), anyhow::Error> {
        Self::MIGRATIONS
            .into_iter()
            .any(|m| database.has_pending_migration(m).unwrap_or(false))
            .then(|| {
                database.exclusive_transaction(|conn| {
                    Self::MIGRATIONS
                        .into_iter()
                        .try_for_each(|m| {
                            conn.run_pending_migrations(m)
                                .map(|_| ()) // discard the result
                                .map_err(|e| anyhow::anyhow!(e))
                        })
                        .map_err(|e| anyhow::anyhow!(e))?;

                    Ok::<(), anyhow::Error>(())
                })
            })
            .transpose()
            .map(|_| ())
    }
}
