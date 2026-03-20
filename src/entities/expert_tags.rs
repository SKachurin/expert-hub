use super::{experts, tags};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "expert_tags")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub expert_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub tag_id: i64,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "experts::Entity",
        from = "Column::ExpertId",
        to = "experts::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Expert,
    #[sea_orm(
        belongs_to = "tags::Entity",
        from = "Column::TagId",
        to = "tags::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Tag,
}

impl Related<experts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Expert.def()
    }
}

impl Related<tags::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tag.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}