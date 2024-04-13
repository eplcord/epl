use sea_orm_migration::prelude::*;
use crate::m20220101_000001_create_user::User;
use crate::m20240412_003408_create_files::File;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(File::Table)
                    .add_column(ColumnDef::new(Alias::new("requested_deletion")).boolean().not_null().default(false))
                    .add_column(ColumnDef::new(Alias::new("uploader")).big_integer().not_null())
                    .add_column(ColumnDef::new(Alias::new("clip")).boolean().not_null().default(false))
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk-file_uploader-user_id")
                            .from_tbl(File::Table)
                            .from_col(Alias::new("uploader"))
                            .to_tbl(User::Table)
                            .to_col(User::Id)
                    )
                    .to_owned()
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(File::Table)
                    .drop_column(Alias::new("requested_deletion"))
                    .drop_column(Alias::new("uploader"))
                    .drop_column(Alias::new("clip"))
                    .drop_foreign_key(Alias::new("fk-file_uploader-user_id"))
                    .to_owned()
            )
            .await
    }
}
