use diesel::{prelude::*, query_builder::*, sqlite::Sqlite};

pub trait BaseEntity: Sized {
    type NewEntityType;
    type Table;

    fn find_by_id(id: i32, conn: &mut SqliteConnection) -> anyhow::Result<Option<Self>>;
    fn insert(entity: Self::NewEntityType, conn: &mut SqliteConnection) -> anyhow::Result<()>;
    fn update<S>(&self, conn: &mut SqliteConnection, changes: S) -> anyhow::Result<()>
    where
        S: AsChangeset<Target = <Self as BaseEntity>::Table> + Send,
        <S as diesel::AsChangeset>::Changeset: QueryFragment<Sqlite> + Send;
}
