use std::{
    fs,
    ops::{Deref, DerefMut},
    path::PathBuf,
    time::Duration,
};

use dirs::home_dir;
use sea_orm::{
    ConnectOptions, ConnectionTrait, Database, DatabaseConnection, EntityTrait, Schema,
    sqlx::database,
};
use sea_orm_migration::SchemaManager;

pub trait DatabaseDriver: Deref<Target = DatabaseConnection> + DerefMut + Sized {
    async fn connect(path: Option<PathBuf>) -> Result<DatabaseConnection, anyhow::Error> {
        // maybe one of the best inline operations i've done to date
        let path = path
            .and_then(|p| p.is_file().then_some(p)) // if the file is not a path, replace with None
            .or_else(|| dirs::home_dir().map(|dir| dir.join(".syncr").join("syncr.db"))) // default
            .ok_or(anyhow::anyhow!(
                // if all else fails, return an error
                "Unable to extract config path, default home directory not found."
            ))?;

        fs::create_dir_all(
            path.parent()
                .ok_or(anyhow::anyhow!("Unable to get parent dir"))?,
        )?;
        fs::File::create(&path)?;

        let mut path = path
            .canonicalize()?
            .to_str()
            .ok_or(anyhow::anyhow!("Unable to convert path to string"))?
            .to_owned();

        println!("DB PATH: {}", path);

        path.push_str("?mode=rwc");
        path.insert_str(0, "sqlite:");

        println!("DB PATH: {}", path);

        let mut opt = ConnectOptions::new(path);
        opt.max_connections(100)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(8))
            .acquire_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8))
            .sqlx_logging(false);
        // .sqlx_logging_level(log::LevelFilter::Info)
        // .set_schema_search_path("my_schema"); // Setting default PostgreSQL schema

        Ok(Database::connect(opt).await?)
    }

    async fn try_create_table<E>(database: &DatabaseConnection, entity: E)
    where
        E: EntityTrait,
    {
        let schema_manager = SchemaManager::new(database);
        if schema_manager
            .has_table(&entity.table_name())
            .await
            .unwrap_or(false)
        {
            return;
        }

        let builder = database.get_database_backend();
        let schema = Schema::new(builder);
        let statement = builder.build(schema.create_table_from_entity(entity).if_not_exists());

        match database.execute(statement).await {
            Ok(_) => println!("Migrated {}", entity.table_name()),
            Err(e) => println!("Error: {}", e),
        }
    }

    async fn new(path: Option<PathBuf>) -> Result<Self, anyhow::Error>;
}
