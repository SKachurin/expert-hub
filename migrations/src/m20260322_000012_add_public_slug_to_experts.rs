use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Experts::Table)
                    .add_column(
                        ColumnDef::new(Experts::PublicSlug)
                            .string_len(255)
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-experts-public-slug")
                    .table(Experts::Table)
                    .col(Experts::PublicSlug)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx-experts-public-slug")
                    .table(Experts::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Experts::Table)
                    .drop_column(Experts::PublicSlug)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Experts {
    Table,
    PublicSlug,
}