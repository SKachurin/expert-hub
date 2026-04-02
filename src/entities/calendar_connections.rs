use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "calendar_connections")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub expert_id: i64,
    pub provider: String,
    pub connection_label: Option<String>,
    pub is_primary: bool,
    pub is_enabled: bool,
    pub connection_status: String,
    pub account_email: Option<String>,
    pub provider_account_id: Option<String>,
    pub provider_user_uri: Option<String>,
    pub provider_organization_uri: Option<String>,
    pub selected_calendar_id: Option<String>,
    pub selected_calendar_name: Option<String>,
    pub selected_calendar_timezone: Option<String>,
    pub selected_event_type_uri: Option<String>,
    pub selected_event_type_name: Option<String>,
    pub selected_scheduling_url: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_expires_at: Option<DateTimeWithTimeZone>,
    pub scopes_json: Option<Json>,
    pub provider_metadata: Option<Json>,
    pub sync_cursor: Option<String>,
    pub last_sync_at: Option<DateTimeWithTimeZone>,
    pub last_sync_status: Option<String>,
    pub last_sync_error: Option<String>,
    pub webhook_signing_secret: Option<String>,
    pub public_link: Option<String>,
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
    #[sea_orm(has_many = "super::bookings::Entity")]
    Bookings,
    #[sea_orm(has_many = "super::calendar_sync_events::Entity")]
    CalendarSyncEvents,
}

impl Related<super::experts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Experts.def()
    }
}

impl Related<super::bookings::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Bookings.def()
    }
}

impl Related<super::calendar_sync_events::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CalendarSyncEvents.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}