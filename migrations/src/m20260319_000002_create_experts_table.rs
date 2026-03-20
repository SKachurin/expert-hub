use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Experts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Experts::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Experts::TelegramId).big_integer().not_null().unique_key())
                    .col(ColumnDef::new(Experts::TelegramUsername).string().not_null().unique_key())
                    .col(ColumnDef::new(Experts::TelegramName).string().not_null())
                    .col(ColumnDef::new(Experts::TelegramBio).text())
                    .col(ColumnDef::new(Experts::TonWalletAddress).string().not_null().unique_key())
                    .col(ColumnDef::new(Experts::CalendarId).big_integer().unique_key())
                    .col(ColumnDef::new(Experts::PhotoUrl).text())
                    .col(
                        ColumnDef::new(Experts::HourlyRate)
                            .decimal_len(10, 2)
                            .not_null()
                            .default(1.00),
                    )
                    .col(ColumnDef::new(Experts::Currency).string_len(8).not_null().default("USD"))
                    .col(
                        ColumnDef::new(Experts::ExpertRating)
                            .decimal_len(3, 2)
                            .not_null()
                            .default(5.00),
                    )
                    .col(ColumnDef::new(Experts::ReviewsCount).integer().not_null().default(0))
                    .col(ColumnDef::new(Experts::Timezone).string().not_null())
                    .col(ColumnDef::new(Experts::WorkingDays).json_binary().not_null())
                    .col(ColumnDef::new(Experts::WorkStartTime).time().not_null())
                    .col(ColumnDef::new(Experts::WorkEndTime).time().not_null())
                    .col(ColumnDef::new(Experts::AllowedSessionDurations).json_binary().not_null())
                    .col(
                        ColumnDef::new(Experts::MinimumNoticeMinutes)
                            .integer()
                            .not_null()
                            .default(60),
                    )
                    .col(
                        ColumnDef::new(Experts::BufferBeforeMinutes)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Experts::BufferAfterMinutes)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Experts::MaxDaysAhead)
                            .integer()
                            .not_null()
                            .default(30),
                    )
                    .col(
                        ColumnDef::new(Experts::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Experts::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_experts_calendar_id")
                            .from(Experts::Table, Experts::CalendarId)
                            .to(CalendarConnections::Table, CalendarConnections::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Experts::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Experts {
    Table,
    Id,
    TelegramId,
    TelegramUsername,
    TelegramName,
    TelegramBio,
    TonWalletAddress,
    CalendarId,
    PhotoUrl,
    HourlyRate,
    Currency,
    ExpertRating,
    ReviewsCount,
    Timezone,
    WorkingDays,
    WorkStartTime,
    WorkEndTime,
    AllowedSessionDurations,
    MinimumNoticeMinutes,
    BufferBeforeMinutes,
    BufferAfterMinutes,
    MaxDaysAhead,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum CalendarConnections {
    Table,
    Id,
}