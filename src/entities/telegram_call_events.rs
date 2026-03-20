use super::{bookings, experts};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "telegram_call_events")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub booking_id: i64,
    pub expert_id: i64,
    pub expert_telegram_id: i64,
    pub customer_telegram_id: i64,
    pub event_type: String,
    pub event_time: DateTimeWithTimeZone,
    pub detected_connection: bool,
    pub payload_json: Json,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "bookings::Entity",
        from = "Column::BookingId",
        to = "bookings::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Booking,
    #[sea_orm(
        belongs_to = "experts::Entity",
        from = "Column::ExpertId",
        to = "experts::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Expert,
}

impl Related<bookings::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Booking.def()
    }
}

impl Related<experts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Expert.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}