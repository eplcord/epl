use sea_orm_migration::prelude::*;
use crate::m20230604_231009_create_message::Message;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Embed::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Embed::Id)
                            .big_integer()
                            .not_null()
                            .primary_key()
                    )
                    .col(
                        ColumnDef::new(Embed::Message)
                            .big_integer()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(Embed::Content)
                            .json()
                            .not_null()
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-embed-message_id")
                            .from(Embed::Table, Embed::Message)
                            .to(Message::Table, Message::Id)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Embed::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Embed {
    Table,
    Id,
    Message,
    Content,
}
