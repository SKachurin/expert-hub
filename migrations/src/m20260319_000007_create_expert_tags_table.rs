use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ExpertTags::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ExpertTags::ExpertId).big_integer().not_null())
                    .col(ColumnDef::new(ExpertTags::TagId).big_integer().not_null())
                    .col(
                        ColumnDef::new(ExpertTags::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(ExpertTags::ExpertId)
                            .col(ExpertTags::TagId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_expert_tags_expert_id")
                            .from(ExpertTags::Table, ExpertTags::ExpertId)
                            .to(Experts::Table, Experts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_expert_tags_tag_id")
                            .from(ExpertTags::Table, ExpertTags::TagId)
                            .to(Tags::Table, Tags::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ExpertTags::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ExpertTags {
    Table,
    ExpertId,
    TagId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Experts {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Tags {
    Table,
    Id,
}