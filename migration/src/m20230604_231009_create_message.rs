use sea_orm_migration::prelude::*;
use crate::m20220101_000001_create_user::User;
use crate::m20230604_223625_create_channel::Channel;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Message::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Message::Id).big_integer().not_null().primary_key())
                    .col(ColumnDef::new(Message::ChannelID).big_integer().not_null())
                    .col(ColumnDef::new(Message::Author).big_integer())
                    .col(ColumnDef::new(Message::Content).string().not_null())
                    .col(ColumnDef::new(Message::Timestamp).date_time().not_null())
                    .col(ColumnDef::new(Message::EditedTimestamp).date_time())
                    .col(ColumnDef::new(Message::TTS).boolean().not_null())
                    .col(ColumnDef::new(Message::MentionEveryone).boolean().not_null())
                    .col(ColumnDef::new(Message::Nonce).string())
                    .col(ColumnDef::new(Message::Pinned).boolean().not_null())
                    // TODO: Fkey this when we have webhooks
                    .col(ColumnDef::new(Message::WebhookID).big_integer())
                    .col(ColumnDef::new(Message::Type).integer().not_null())
                    .col(ColumnDef::new(Message::ApplicationID).big_integer())
                    .col(ColumnDef::new(Message::MessageReference).big_integer())
                    .col(ColumnDef::new(Message::Flags).integer())

                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-message_channel_id-channel_id")
                            .from(Message::Table, Message::ChannelID)
                            .to(Channel::Table, Channel::Id)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-message_author-user_id")
                            .from(Message::Table, Message::Author)
                            .to(User::Table, User::Id)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-message_message_reference-message_id")
                            .from(Message::Table, Message::MessageReference)
                            .to(Message::Table, Message::Id)
                    )

                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Message::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum Message {
    Table,
    Id,
    ChannelID,
    Author,
    Content,
    Timestamp,
    EditedTimestamp,
    TTS,
    MentionEveryone,
    Nonce,
    Pinned,
    WebhookID,
    Type,
    ApplicationID,
    MessageReference,
    Flags,
}
