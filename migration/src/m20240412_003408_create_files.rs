use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(File::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(File::Id)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(File::UploadId).string())
                    .col(ColumnDef::new(File::Pending).boolean().not_null())
                    .col(ColumnDef::new(File::Type).integer().not_null())
                    .col(ColumnDef::new(File::ContentType).string())
                    .col(ColumnDef::new(File::Size).big_integer().not_null())
                    .col(ColumnDef::new(File::Name).string().not_null())
                    .col(ColumnDef::new(File::Width).big_integer())
                    .col(ColumnDef::new(File::Height).big_integer())
                    .col(ColumnDef::new(File::Timestamp).date_time().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(File::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum File {
    Table,
    Id,
    UploadId,
    Pending,
    Type,
    ContentType,
    Size,
    Name,
    Width,
    Height,
    Timestamp
}
