use crate::schema::predictor_saves;
use diesel::{prelude::*, query_builder::QueryFragment, sqlite::Sqlite};

use super::base::BaseEntity;

#[derive(Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = predictor_saves)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct PredictorSave {
    pub id: i32,

    pub save: Vec<u8>,

    pub created_at: chrono::NaiveDateTime,

    pub updated_at: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = predictor_saves)]
pub struct NewPredictorSave {
    pub save: Vec<u8>,

    pub created_at: chrono::NaiveDateTime,

    pub updated_at: chrono::NaiveDateTime,
}

impl Default for NewPredictorSave {
    fn default() -> Self {
        Self {
            save: Vec::new(),
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        }
    }
}

impl BaseEntity for PredictorSave {
    type NewEntityType = NewPredictorSave;
    type Table = predictor_saves::table;

    fn find_by_id(id_: i32, conn: &mut SqliteConnection) -> anyhow::Result<Option<Self>> {
        use crate::schema::predictor_saves::dsl::*;

        Ok(predictor_saves
            .filter(id.eq(id_))
            .first::<Self>(conn)
            .optional()?)
    }

    fn insert(entity: NewPredictorSave, conn: &mut SqliteConnection) -> anyhow::Result<()> {
        use crate::schema::predictor_saves;

        diesel::insert_into(predictor_saves::table)
            .values(entity)
            .execute(conn)?;

        Ok(())
    }

    fn update<S>(&self, conn: &mut SqliteConnection, changes: S) -> anyhow::Result<()>
    where
        S: AsChangeset<Target = <Self as BaseEntity>::Table> + Send,
        <S as diesel::AsChangeset>::Changeset: QueryFragment<Sqlite> + Send,
    {
        conn.transaction(|conn| {
            diesel::update(&self).set(changes).execute(conn)?;

            Ok(())
        })
    }
}

impl PredictorSave {
    pub fn quick_insert(save: Vec<u8>, conn: &mut SqliteConnection) {
        let entity = NewPredictorSave {
            save,
            ..Default::default()
        };

        Self::insert(entity, conn).unwrap();
    }
}
