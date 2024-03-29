use sea_orm_migration::prelude::*;
use crate::m20230604_223625_create_channel::Channel;
use crate::m20230604_231009_create_message::Message;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Pin::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Pin::Channel)
                            .big_integer()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(Pin::Message)
                            .big_integer()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(Pin::Timestamp)
                            .date_time()
                            .not_null()
                    )
                    .primary_key(
                        Index::create()
                            .col(Pin::Channel)
                            .col(Pin::Message)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_pin_channel_id")
                            .from(Pin::Table, Pin::Channel)
                            .to(Channel::Table, Channel::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_pin_message_id")
                            .from(Pin::Table, Pin::Message)
                            .to(Message::Table, Message::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Pin::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Pin {
    Table,
    Channel,
    Message,
    Timestamp
}
