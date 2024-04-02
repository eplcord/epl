use sea_orm_migration::prelude::*;
use crate::m20220101_000001_create_user::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AprilFools2024::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AprilFools2024::Id)
                            .big_integer()
                            .not_null()
                            .primary_key()
                    )
                    .col(
                        ColumnDef::new(AprilFools2024::User)
                            .big_integer()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(AprilFools2024::Item)
                            .big_integer()
                            .not_null()
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-aprilfools2024-user_id")
                            .from(AprilFools2024::Table, AprilFools2024::User)
                            .to(User::Table, User::Id)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AprilFools2024::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AprilFools2024 {
    Table,
    Id,
    User,
    Item,
}
