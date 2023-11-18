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
                    .table(Mention::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Mention::Message)
                            .big_integer()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(Mention::User)
                            .big_integer()
                            .not_null()
                    )
                    .primary_key(
                        Index::create()
                            .col(Mention::Message)
                            .col(Mention::User)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_mention_message_id")
                            .from(Mention::Table, Mention::Message)
                            .to(Message::Table, Message::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_mention_user_id")
                            .from(Mention::Table, Mention::User)
                            .to(User::Table, User::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Mention::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Mention {
    Table,
    Message,
    User,
}
