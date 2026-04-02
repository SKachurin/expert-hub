use sea_orm::entity::prelude::*;
use rust_decimal::Decimal;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "bookings")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub expert_id: i64,
    pub calendar_connection_id: Option<i64>,
    pub provider_used: Option<String>,
    pub requested_by_telegram_id: i64,
    pub requested_by_username: Option<String>,
    pub requested_by_display_name: Option<String>,
    pub requested_by_ton_wallet: Option<String>,
    pub expert_timezone: String,
    pub requested_duration_minutes: i32,
    pub hourly_rate_snapshot: Decimal,
    pub amount_quoted: Decimal,
    pub currency: String,
    pub slot_start: DateTimeWithTimeZone,
    pub slot_end: DateTimeWithTimeZone,
    pub status: String,
    pub expert_confirmed_at: Option<DateTimeWithTimeZone>,
    pub expert_rejected_at: Option<DateTimeWithTimeZone>,
    pub rejected_reason: Option<String>,
    pub payment_required_by: Option<DateTimeWithTimeZone>,
    pub hold_expires_at: Option<DateTimeWithTimeZone>,
    pub expires_at: DateTimeWithTimeZone,
    pub external_booking_ref: Option<String>,
    pub external_event_id: Option<String>,
    pub external_event_url: Option<String>,
    pub external_meeting_url: Option<String>,
    pub external_join_url: Option<String>,
    pub external_cancel_url: Option<String>,
    pub external_sync_status: String,
    pub external_synced_at: Option<DateTimeWithTimeZone>,
    pub external_sync_error: Option<String>,
    pub session_started_at: Option<DateTimeWithTimeZone>,
    pub session_connected_at: Option<DateTimeWithTimeZone>,
    pub session_ended_at: Option<DateTimeWithTimeZone>,
    pub outcome_source: Option<String>,
    pub metadata: Option<Json>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::experts::Entity",
        from = "Column::ExpertId",
        to = "super::experts::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Experts,
    #[sea_orm(
        belongs_to = "super::calendar_connections::Entity",
        from = "Column::CalendarConnectionId",
        to = "super::calendar_connections::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    CalendarConnections,
    #[sea_orm(has_many = "super::payments::Entity")]
    Payments,
}

impl Related<super::experts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Experts.def()
    }
}

impl Related<super::calendar_connections::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CalendarConnections.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}