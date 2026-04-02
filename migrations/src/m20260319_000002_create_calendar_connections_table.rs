use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CalendarConnections::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CalendarConnections::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(CalendarConnections::ExpertId).big_integer().not_null())
                    .col(ColumnDef::new(CalendarConnections::Provider).string_len(32).not_null())
                    .col(ColumnDef::new(CalendarConnections::ConnectionLabel).string_len(255))
                    .col(
                        ColumnDef::new(CalendarConnections::IsPrimary)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(CalendarConnections::IsEnabled)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(CalendarConnections::ConnectionStatus)
                            .string_len(32)
                            .not_null()
                            .default("pending"),
                    )
                    .col(ColumnDef::new(CalendarConnections::AccountEmail).string_len(255))
                    .col(ColumnDef::new(CalendarConnections::ProviderAccountId).string_len(255))
                    .col(ColumnDef::new(CalendarConnections::ProviderUserUri).string_len(512))
                    .col(ColumnDef::new(CalendarConnections::ProviderOrganizationUri).string_len(512))
                    .col(ColumnDef::new(CalendarConnections::SelectedCalendarId).string_len(255))
                    .col(ColumnDef::new(CalendarConnections::SelectedCalendarName).string_len(255))
                    .col(ColumnDef::new(CalendarConnections::SelectedCalendarTimezone).string_len(64))
                    .col(ColumnDef::new(CalendarConnections::SelectedEventTypeUri).string_len(512))
                    .col(ColumnDef::new(CalendarConnections::SelectedEventTypeName).string_len(255))
                    .col(ColumnDef::new(CalendarConnections::SelectedSchedulingUrl).string_len(1024))
                    .col(ColumnDef::new(CalendarConnections::AccessToken).text())
                    .col(ColumnDef::new(CalendarConnections::RefreshToken).text())
                    .col(
                        ColumnDef::new(CalendarConnections::TokenExpiresAt)
                            .timestamp_with_time_zone(),
                    )
                    .col(ColumnDef::new(CalendarConnections::ScopesJson).json_binary())
                    .col(ColumnDef::new(CalendarConnections::ProviderMetadata).json_binary())
                    .col(ColumnDef::new(CalendarConnections::SyncCursor).text())
                    .col(ColumnDef::new(CalendarConnections::LastSyncAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(CalendarConnections::LastSyncStatus).string_len(32))
                    .col(ColumnDef::new(CalendarConnections::LastSyncError).text())
                    .col(ColumnDef::new(CalendarConnections::WebhookSigningSecret).text())
                    .col(ColumnDef::new(CalendarConnections::PublicLink).text())
                    .col(
                        ColumnDef::new(CalendarConnections::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CalendarConnections::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_calendar_connections_expert_id")
                            .from(CalendarConnections::Table, CalendarConnections::ExpertId)
                            .to(Experts::Table, Experts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_calendar_connections_expert_id")
                    .table(CalendarConnections::Table)
                    .col(CalendarConnections::ExpertId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_calendar_connections_provider")
                    .table(CalendarConnections::Table)
                    .col(CalendarConnections::Provider)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_calendar_connections_primary")
                    .table(CalendarConnections::Table)
                    .col(CalendarConnections::ExpertId)
                    .col(CalendarConnections::IsPrimary)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_calendar_connections_enabled")
                    .table(CalendarConnections::Table)
                    .col(CalendarConnections::ExpertId)
                    .col(CalendarConnections::IsEnabled)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CalendarConnections::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum CalendarConnections {
    Table,
    Id,
    ExpertId,
    Provider,
    ConnectionLabel,
    IsPrimary,
    IsEnabled,
    ConnectionStatus,
    AccountEmail,
    ProviderAccountId,
    ProviderUserUri,
    ProviderOrganizationUri,
    SelectedCalendarId,
    SelectedCalendarName,
    SelectedCalendarTimezone,
    SelectedEventTypeUri,
    SelectedEventTypeName,
    SelectedSchedulingUrl,
    AccessToken,
    RefreshToken,
    TokenExpiresAt,
    ScopesJson,
    ProviderMetadata,
    SyncCursor,
    LastSyncAt,
    LastSyncStatus,
    LastSyncError,
    WebhookSigningSecret,
    PublicLink,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Experts {
    Table,
    Id,
}