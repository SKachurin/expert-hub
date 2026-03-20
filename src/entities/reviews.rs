use super::experts;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "reviews")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub review_text: Option<String>,
    pub expert_id: i64,
    pub author_telegram_id: i64,
    pub author_telegram_username: String,
    pub author_telegram_name: String,
    pub review_rating: i16,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
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
}

impl Related<experts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Expert.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}