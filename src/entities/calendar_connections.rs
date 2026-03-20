use super::experts;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "calendar_connections")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub provider: String,
    pub link: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "experts::Entity")]
    Experts,
}

impl Related<experts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Experts.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}