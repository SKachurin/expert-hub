use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CalendarSyncEvents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CalendarSyncEvents::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(CalendarSyncEvents::CalendarConnectionId).big_integer().not_null())
                    .col(ColumnDef::new(CalendarSyncEvents::SyncType).string_len(32).not_null())
                    .col(ColumnDef::new(CalendarSyncEvents::SyncStatus).string_len(32).not_null())
                    .col(ColumnDef::new(CalendarSyncEvents::StartedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(CalendarSyncEvents::FinishedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(CalendarSyncEvents::RemoteCursorBefore).text())
                    .col(ColumnDef::new(CalendarSyncEvents::RemoteCursorAfter).text())
                    .col(ColumnDef::new(CalendarSyncEvents::ItemsRead).integer())
                    .col(ColumnDef::new(CalendarSyncEvents::ItemsChanged).integer())
                    .col(ColumnDef::new(CalendarSyncEvents::ItemsDeleted).integer())
                    .col(ColumnDef::new(CalendarSyncEvents::ErrorMessage).text())
                    .col(ColumnDef::new(CalendarSyncEvents::Payload).json_binary())
                    .col(ColumnDef::new(CalendarSyncEvents::CreatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_calendar_sync_events_calendar_connection_id")
                            .from(
                                CalendarSyncEvents::Table,
                                CalendarSyncEvents::CalendarConnectionId,
                            )
                            .to(CalendarConnections::Table, CalendarConnections::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_calendar_sync_events_connection_id")
                    .table(CalendarSyncEvents::Table)
                    .col(CalendarSyncEvents::CalendarConnectionId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_calendar_sync_events_status")
                    .table(CalendarSyncEvents::Table)
                    .col(CalendarSyncEvents::SyncStatus)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CalendarSyncEvents::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum CalendarSyncEvents {
    Table,
    Id,
    CalendarConnectionId,
    SyncType,
    SyncStatus,
    StartedAt,
    FinishedAt,
    RemoteCursorBefore,
    RemoteCursorAfter,
    ItemsRead,
    ItemsChanged,
    ItemsDeleted,
    ErrorMessage,
    Payload,
    CreatedAt,
}

#[derive(DeriveIden)]
enum CalendarConnections {
    Table,
    Id,
}