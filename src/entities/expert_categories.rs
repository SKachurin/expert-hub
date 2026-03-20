use super::{categories, experts};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "expert_categories")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub expert_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub category_id: i64,
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
        belongs_to = "categories::Entity",
        from = "Column::CategoryId",
        to = "categories::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Category,
}

impl Related<experts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Expert.def()
    }
}

impl Related<categories::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Category.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}