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
                    .col(ColumnDef::new(Bookings::CalendarId).big_integer().not_null())
                    .col(
                        ColumnDef::new(Bookings::RequestedByTelegramId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Bookings::SlotStart)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Bookings::SlotEnd)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Bookings::Status).string().not_null().default("pending"))
                    .col(
                        ColumnDef::new(Bookings::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
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
                            .name("fk_bookings_calendar_id")
                            .from(Bookings::Table, Bookings::CalendarId)
                            .to(CalendarConnections::Table, CalendarConnections::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
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
    CalendarId,
    RequestedByTelegramId,
    SlotStart,
    SlotEnd,
    Status,
    ExpiresAt,
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