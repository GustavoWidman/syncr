use linfa_trees::DecisionTree;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "predictor_saves")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    pub save: Vec<u8>,

    #[sea_orm(created_at, column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime,

    #[sea_orm(updated_at, column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
