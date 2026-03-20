use super::expert_categories;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "categories")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub name: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "expert_categories::Entity")]
    ExpertCategories,
}

impl Related<expert_categories::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ExpertCategories.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}