use sea_orm::entity::prelude::*;
use rust_decimal::Decimal;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "experts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub telegram_id: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub display_name: String,
    pub telegram_bio: Option<String>,
    pub photo_url: Option<String>,
    pub ton_wallet_address: String,
    pub timezone: String,
    pub hourly_rate: Decimal,
    pub currency: String,
    pub working_days: Json,
    pub work_start_time: Time,
    pub work_end_time: Time,
    pub allowed_session_durations: Json,
    pub minimum_notice_minutes: i32,
    pub buffer_before_minutes: i32,
    pub buffer_after_minutes: i32,
    pub max_days_ahead: i32,
    pub calendar_conflict_mode: String,
    pub booking_target_strategy: String,
    pub is_active: bool,
    pub is_bookable: bool,
    pub expert_rating: Decimal,
    pub reviews_count: i32,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::bookings::Entity")]
    Bookings,
    #[sea_orm(has_many = "super::calendar_connections::Entity")]
    CalendarConnections,
    #[sea_orm(has_many = "super::reviews::Entity")]
    Reviews,
    #[sea_orm(has_many = "super::expert_tags::Entity")]
    ExpertTags,
    #[sea_orm(has_many = "super::expert_categories::Entity")]
    ExpertCategories,
}

impl Related<super::bookings::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Bookings.def()
    }
}

impl Related<super::calendar_connections::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CalendarConnections.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}