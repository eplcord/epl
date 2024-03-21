use sea_orm_migration::prelude::*;
use crate::m20220101_000001_create_user::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .add_column(ColumnDef::new(Alias::new("display_name")).text())
                    .add_column(ColumnDef::new(Alias::new("legacy_name")).text())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .drop_column(Alias::new("display_name"))
                    .drop_column(Alias::new("legacy_name"))
                    .to_owned(),
            )
            .await
    }
}