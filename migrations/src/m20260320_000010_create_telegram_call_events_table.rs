use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TelegramCallEvents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TelegramCallEvents::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(TelegramCallEvents::BookingId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TelegramCallEvents::ExpertId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TelegramCallEvents::ExpertTelegramId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TelegramCallEvents::CustomerTelegramId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TelegramCallEvents::EventType)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TelegramCallEvents::EventTime)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TelegramCallEvents::DetectedConnection)
                            .boolean()
                            .not_null(),
                    )
                    .col(ColumnDef::new(TelegramCallEvents::PayloadJson).json())
                    .col(
                        ColumnDef::new(TelegramCallEvents::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_telegram_call_events_booking_id")
                    .from(TelegramCallEvents::Table, TelegramCallEvents::BookingId)
                    .to(Bookings::Table, Bookings::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_telegram_call_events_expert_id")
                    .from(TelegramCallEvents::Table, TelegramCallEvents::ExpertId)
                    .to(Experts::Table, Experts::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_telegram_call_events_booking_id")
                    .table(TelegramCallEvents::Table)
                    .col(TelegramCallEvents::BookingId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_telegram_call_events_expert_id")
                    .table(TelegramCallEvents::Table)
                    .col(TelegramCallEvents::ExpertId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_telegram_call_events_expert_telegram_id")
                    .table(TelegramCallEvents::Table)
                    .col(TelegramCallEvents::ExpertTelegramId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_telegram_call_events_customer_telegram_id")
                    .table(TelegramCallEvents::Table)
                    .col(TelegramCallEvents::CustomerTelegramId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_telegram_call_events_event_type")
                    .table(TelegramCallEvents::Table)
                    .col(TelegramCallEvents::EventType)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TelegramCallEvents::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TelegramCallEvents {
    Table,
    Id,
    BookingId,
    ExpertId,
    ExpertTelegramId,
    CustomerTelegramId,
    EventType,
    EventTime,
    DetectedConnection,
    PayloadJson,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Bookings {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Experts {
    Table,
    Id,
}