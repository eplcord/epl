use sea_orm_migration::prelude::*;
use crate::m20230604_231009_create_message::Message;
use crate::m20240412_003408_create_files::File;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager
            .create_table(
                Table::create()
                    .table(MessageAttachment::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(MessageAttachment::Message).big_integer().not_null())
                    .col(ColumnDef::new(MessageAttachment::File).big_integer().not_null())
                    .primary_key(Index::create().col(MessageAttachment::Message).col(MessageAttachment::File))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_message-attachment_message-id")
                            .from(MessageAttachment::Table, MessageAttachment::Message)
                            .to(Message::Table, Message::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_message-attachment_file-id")
                            .from(MessageAttachment::Table,MessageAttachment::File)
                            .to(File::Table, File::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MessageAttachment::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum MessageAttachment {
    Table,
    Message,
    File,
}
