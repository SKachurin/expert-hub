use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Bookings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Bookings::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Bookings::ExpertId).big_integer().not_null())
                    .col(ColumnDef::new(Bookings::CalendarConnectionId).big_integer())
                    .col(ColumnDef::new(Bookings::ProviderUsed).string_len(32))
                    .col(ColumnDef::new(Bookings::RequestedByTelegramId).big_integer().not_null())
                    .col(ColumnDef::new(Bookings::RequestedByUsername).string_len(255))
                    .col(ColumnDef::new(Bookings::RequestedByDisplayName).string_len(255))
                    .col(ColumnDef::new(Bookings::RequestedByTonWallet).string_len(255))
                    .col(ColumnDef::new(Bookings::ExpertTimezone).string_len(64).not_null())
                    .col(ColumnDef::new(Bookings::RequestedDurationMinutes).integer().not_null())
                    .col(ColumnDef::new(Bookings::HourlyRateSnapshot).decimal_len(12, 2).not_null())
                    .col(ColumnDef::new(Bookings::AmountQuoted).decimal_len(12, 2).not_null())
                    .col(ColumnDef::new(Bookings::Currency).string_len(8).not_null())
                    .col(ColumnDef::new(Bookings::SlotStart).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Bookings::SlotEnd).timestamp_with_time_zone().not_null())
                    .col(
                        ColumnDef::new(Bookings::Status)
                            .string_len(32)
                            .not_null()
                            .default("requested"),
                    )
                    .col(ColumnDef::new(Bookings::ExpertConfirmedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Bookings::ExpertRejectedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Bookings::RejectedReason).text())
                    .col(ColumnDef::new(Bookings::PaymentRequiredBy).timestamp_with_time_zone())
                    .col(ColumnDef::new(Bookings::HoldExpiresAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Bookings::ExpiresAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Bookings::ExternalBookingRef).string_len(255))
                    .col(ColumnDef::new(Bookings::ExternalEventId).string_len(255))
                    .col(ColumnDef::new(Bookings::ExternalEventUrl).text())
                    .col(ColumnDef::new(Bookings::ExternalMeetingUrl).text())
                    .col(ColumnDef::new(Bookings::ExternalJoinUrl).text())
                    .col(ColumnDef::new(Bookings::ExternalCancelUrl).text())
                    .col(
                        ColumnDef::new(Bookings::ExternalSyncStatus)
                            .string_len(32)
                            .not_null()
                            .default("not_synced"),
                    )
                    .col(ColumnDef::new(Bookings::ExternalSyncedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Bookings::ExternalSyncError).text())
                    .col(ColumnDef::new(Bookings::SessionStartedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Bookings::SessionConnectedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Bookings::SessionEndedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Bookings::OutcomeSource).string_len(64))
                    .col(ColumnDef::new(Bookings::Metadata).json_binary())
                    .col(
                        ColumnDef::new(Bookings::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Bookings::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_bookings_expert_id")
                            .from(Bookings::Table, Bookings::ExpertId)
                            .to(Experts::Table, Experts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_bookings_calendar_connection_id")
                            .from(Bookings::Table, Bookings::CalendarConnectionId)
                            .to(CalendarConnections::Table, CalendarConnections::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        for index in [
            Index::create()
                .name("idx_bookings_expert_id")
                .table(Bookings::Table)
                .col(Bookings::ExpertId)
                .to_owned(),
            Index::create()
                .name("idx_bookings_calendar_connection_id")
                .table(Bookings::Table)
                .col(Bookings::CalendarConnectionId)
                .to_owned(),
            Index::create()
                .name("idx_bookings_status")
                .table(Bookings::Table)
                .col(Bookings::Status)
                .to_owned(),
            Index::create()
                .name("idx_bookings_slot_start")
                .table(Bookings::Table)
                .col(Bookings::SlotStart)
                .to_owned(),
            Index::create()
                .name("idx_bookings_requested_by_telegram_id")
                .table(Bookings::Table)
                .col(Bookings::RequestedByTelegramId)
                .to_owned(),
        ] {
            manager.create_index(index).await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Bookings::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Bookings {
    Table,
    Id,
    ExpertId,
    CalendarConnectionId,
    ProviderUsed,
    RequestedByTelegramId,
    RequestedByUsername,
    RequestedByDisplayName,
    RequestedByTonWallet,
    ExpertTimezone,
    RequestedDurationMinutes,
    HourlyRateSnapshot,
    AmountQuoted,
    Currency,
    SlotStart,
    SlotEnd,
    Status,
    ExpertConfirmedAt,
    ExpertRejectedAt,
    RejectedReason,
    PaymentRequiredBy,
    HoldExpiresAt,
    ExpiresAt,
    ExternalBookingRef,
    ExternalEventId,
    ExternalEventUrl,
    ExternalMeetingUrl,
    ExternalJoinUrl,
    ExternalCancelUrl,
    ExternalSyncStatus,
    ExternalSyncedAt,
    ExternalSyncError,
    SessionStartedAt,
    SessionConnectedAt,
    SessionEndedAt,
    OutcomeSource,
    Metadata,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Experts {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum CalendarConnections {
    Table,
    Id,
}