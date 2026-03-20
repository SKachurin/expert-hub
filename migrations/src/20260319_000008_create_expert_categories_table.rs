use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ExpertCategories::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ExpertCategories::ExpertId).big_integer().not_null())
                    .col(ColumnDef::new(ExpertCategories::CategoryId).big_integer().not_null())
                    .col(
                        ColumnDef::new(ExpertCategories::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(ExpertCategories::ExpertId)
                            .col(ExpertCategories::CategoryId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_expert_categories_expert_id")
                            .from(ExpertCategories::Table, ExpertCategories::ExpertId)
                            .to(Experts::Table, Experts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_expert_categories_category_id")
                            .from(ExpertCategories::Table, ExpertCategories::CategoryId)
                            .to(Categories::Table, Categories::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ExpertCategories::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ExpertCategories {
    Table,
    ExpertId,
    CategoryId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Experts {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Categories {
    Table,
    Id,
}