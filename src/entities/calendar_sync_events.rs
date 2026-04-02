use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "calendar_sync_events")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub calendar_connection_id: i64,
    pub sync_type: String,
    pub sync_status: String,
    pub started_at: DateTimeWithTimeZone,
    pub finished_at: Option<DateTimeWithTimeZone>,
    pub remote_cursor_before: Option<String>,
    pub remote_cursor_after: Option<String>,
    pub items_read: Option<i32>,
    pub items_changed: Option<i32>,
    pub items_deleted: Option<i32>,
    pub error_message: Option<String>,
    pub payload: Option<Json>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::calendar_connections::Entity",
        from = "Column::CalendarConnectionId",
        to = "super::calendar_connections::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    CalendarConnections,
}

impl Related<super::calendar_connections::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CalendarConnections.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}