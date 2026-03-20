use super::{calendar_connections, experts, payments, telegram_call_events};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "bookings")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub expert_id: i64,
    pub calendar_id: i64,
    pub requested_by_telegram_id: i64,
    pub slot_start: DateTimeWithTimeZone,
    pub slot_end: DateTimeWithTimeZone,
    pub status: String,
    pub expires_at: DateTimeWithTimeZone,
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
    #[sea_orm(
        belongs_to = "calendar_connections::Entity",
        from = "Column::CalendarId",
        to = "calendar_connections::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    CalendarConnection,
    #[sea_orm(has_many = "payments::Entity")]
    Payments,
    #[sea_orm(has_many = "telegram_call_events::Entity")]
    TelegramCallEvents,
}

impl Related<experts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Expert.def()
    }
}

impl Related<calendar_connections::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CalendarConnection.def()
    }
}

impl Related<payments::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Payments.def()
    }
}

impl Related<telegram_call_events::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TelegramCallEvents.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}