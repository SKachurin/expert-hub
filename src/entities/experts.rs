use super::{
    bookings, calendar_connections, expert_categories, expert_tags, payments, reviews,
    telegram_call_events,
};
use rust_decimal::Decimal;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "experts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub telegram_id: i64,
    pub telegram_username: String,
    pub telegram_name: String,
    pub telegram_bio: Option<String>,
    pub ton_wallet_address: String,
    pub calendar_id: Option<i64>,
    pub photo_url: Option<String>,
    pub hourly_rate: Decimal,
    pub currency: String,
    pub expert_rating: Decimal,
    pub reviews_count: i32,
    pub timezone: String,
    pub working_days: Json,
    pub work_start_time: Time,
    pub work_end_time: Time,
    pub allowed_session_durations: Json,
    pub minimum_notice_minutes: i32,
    pub buffer_before_minutes: i32,
    pub buffer_after_minutes: i32,
    pub max_days_ahead: i32,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "calendar_connections::Entity",
        from = "Column::CalendarId",
        to = "calendar_connections::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    CalendarConnection,
    #[sea_orm(has_many = "reviews::Entity")]
    Reviews,
    #[sea_orm(has_many = "expert_tags::Entity")]
    ExpertTags,
    #[sea_orm(has_many = "expert_categories::Entity")]
    ExpertCategories,
    #[sea_orm(has_many = "bookings::Entity")]
    Bookings,
    #[sea_orm(has_many = "payments::Entity")]
    Payments,
    #[sea_orm(has_many = "telegram_call_events::Entity")]
    TelegramCallEvents,
}

impl Related<calendar_connections::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CalendarConnection.def()
    }
}

impl Related<reviews::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Reviews.def()
    }
}

impl Related<expert_tags::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ExpertTags.def()
    }
}

impl Related<expert_categories::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ExpertCategories.def()
    }
}

impl Related<bookings::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Bookings.def()
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