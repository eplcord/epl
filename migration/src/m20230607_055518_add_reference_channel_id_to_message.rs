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
                    .table(Message::Table)
                    .add_column(
                        ColumnDef::new(Alias::new("reference_channel_id"))
                            .big_integer()
                    )
                    .add_foreign_key(
                        TableForeignKey::new()
                            .name("fk-message_reference_channel_id-channel_id")
                            .from_tbl(Message::Table)
                            .from_col(Alias::new("reference_channel_id"))
                            .to_tbl(Channel::Table)
                            .to_col(Channel::Id)
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
                    .table(Message::Table)
                    .drop_column(Alias::new("reference_channel_id"))
                    .drop_foreign_key(Alias::new("fk-message_reference_channel_id-channel_id"))
                    .to_owned()
            )
            .await
    }
}