use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Payments::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Payments::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Payments::BookingId).big_integer().not_null())
                    .col(ColumnDef::new(Payments::ExpertId).big_integer().not_null())
                    .col(
                        ColumnDef::new(Payments::CustomerTelegramId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Payments::Amount)
                            .decimal()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Payments::Currency)
                            .string_len(8)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Payments::Status)
                            .string_len(32)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Payments::TonWalletCustomer).string())
                    .col(ColumnDef::new(Payments::TonWalletExpert).string())
                    .col(ColumnDef::new(Payments::ContractAddress).string())
                    .col(ColumnDef::new(Payments::TransactionRef).string())
                    .col(
                        ColumnDef::new(Payments::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Payments::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_payments_booking_id")
                    .from(Payments::Table, Payments::BookingId)
                    .to(Bookings::Table, Bookings::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_payments_expert_id")
                    .from(Payments::Table, Payments::ExpertId)
                    .to(Experts::Table, Experts::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_payments_booking_id")
                    .table(Payments::Table)
                    .col(Payments::BookingId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_payments_expert_id")
                    .table(Payments::Table)
                    .col(Payments::ExpertId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_payments_customer_telegram_id")
                    .table(Payments::Table)
                    .col(Payments::CustomerTelegramId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_payments_status")
                    .table(Payments::Table)
                    .col(Payments::Status)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Payments::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Payments {
    Table,
    Id,
    BookingId,
    ExpertId,
    CustomerTelegramId,
    Amount,
    Currency,
    Status,
    TonWalletCustomer,
    TonWalletExpert,
    ContractAddress,
    TransactionRef,
    CreatedAt,
    UpdatedAt,
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