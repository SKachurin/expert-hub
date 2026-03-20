use super::expert_tags;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "tags")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub value: String,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "expert_tags::Entity")]
    ExpertTags,
}

impl Related<expert_tags::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ExpertTags.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}