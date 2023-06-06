use sea_orm_migration::prelude::*;
use crate::m20230604_223625_create_channel::Channel;
use crate::m20230604_231009_create_message::Message;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Channel::Table)
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk-channel_last_message_id-message_id")
                            .from_tbl(Channel::Table)
                            .from_col(Channel::LastMessageID)
                            .to_tbl(Message::Table)
                            .to_col(Message::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Channel::Table)
                    .drop_foreign_key(Alias::new("fk-channel_last_message_id-message_id"))
                    .to_owned()
            )
            .await
    }
}