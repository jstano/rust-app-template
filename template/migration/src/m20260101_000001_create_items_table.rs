//! Starter migration for the `items` table backing the `Item` entity in
//! `{{ crate_prefix }}-domain`. Not yet wired into `{{ crate_prefix }}-persistence`
//! (which uses an in-memory store) — switch the persistence adapter to `stano-seaorm`
//! and point it at this table when you're ready for real persistence.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Item::Table)
                    .col(ColumnDef::new(Item::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Item::Name).text().not_null())
                    .col(
                        ColumnDef::new(Item::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Item::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Item {
    Table,
    Id,
    Name,
    CreatedAt,
}
