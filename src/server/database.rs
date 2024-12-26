use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use sea_orm::DatabaseConnection;

use crate::data::entities::predictor::Entity as Predictor;

use crate::data::DatabaseDriver;

pub struct ServerDatabase {
    inner: DatabaseConnection,
}

impl DatabaseDriver for ServerDatabase {
    async fn new(path: Option<PathBuf>) -> Result<Self, anyhow::Error> {
        let database = Self::connect(path).await?;

        Self::try_create_table(&database, Predictor).await;

        Ok(Self { inner: database })
    }
}

impl Deref for ServerDatabase {
    type Target = DatabaseConnection;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ServerDatabase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
