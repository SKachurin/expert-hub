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
                    .col(ColumnDef::new(CalendarConnections::Provider).string().not_null())
                    .col(ColumnDef::new(CalendarConnections::Link).text())
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
    Provider,
    Link,
    CreatedAt,
    UpdatedAt,
}