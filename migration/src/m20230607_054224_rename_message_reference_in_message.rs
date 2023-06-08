use crate::m20230604_231009_create_message::Message;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Message::Table)
                    .rename_column(
                        Alias::new("message_reference"),
                        Alias::new("reference_message_id"),
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
                    .rename_column(
                        Alias::new("reference_message_id"),
                        Alias::new("message_reference"),
                    )
                    .to_owned(),
            )
            .await
    }
}
