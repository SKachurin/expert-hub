use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Reviews::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Reviews::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Reviews::ReviewText).text())
                    .col(ColumnDef::new(Reviews::ExpertId).big_integer().not_null())
                    .col(ColumnDef::new(Reviews::AuthorTelegramId).big_integer().not_null())
                    .col(ColumnDef::new(Reviews::AuthorTelegramUsername).string().not_null())
                    .col(ColumnDef::new(Reviews::AuthorTelegramName).string().not_null())
                    .col(ColumnDef::new(Reviews::ReviewRating).small_integer().not_null())
                    .col(
                        ColumnDef::new(Reviews::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Reviews::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_reviews_expert_id")
                            .from(Reviews::Table, Reviews::ExpertId)
                            .to(Experts::Table, Experts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Reviews::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Reviews {
    Table,
    Id,
    ReviewText,
    ExpertId,
    AuthorTelegramId,
    AuthorTelegramUsername,
    AuthorTelegramName,
    ReviewRating,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Experts {
    Table,
    Id,
}