use sea_orm_migration::prelude::*;
use crate::m20220101_000001_create_user::User;
use crate::m20230604_231009_create_message::Message;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Reaction::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Reaction::User).big_integer().not_null())
                    .col(ColumnDef::new(Reaction::Message).big_integer().not_null())
                    .col(ColumnDef::new(Reaction::Emoji).text().not_null())
                    .col(ColumnDef::new(Reaction::Burst).boolean().not_null())
                    .primary_key(Index::create().col(Reaction::User).col(Reaction::Message).col(Reaction::Emoji))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_reaction-user_message-id")
                            .from(Reaction::Table, Reaction::User)
                            .to(User::Table, User::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_reaction-message_file-id")
                            .from(Reaction::Table, Reaction::Message)
                            .to(Message::Table, Message::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Reaction::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Reaction {
    Table,
    User,
    Emoji,
    Message,
    Burst,
}
