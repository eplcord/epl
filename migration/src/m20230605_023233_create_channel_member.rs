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
                    .table(ChannelMember::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ChannelMember::Channel).big_integer().not_null())
                    .col(ColumnDef::new(ChannelMember::User).big_integer().not_null())

                    .primary_key(Index::create().col(ChannelMember::Channel).col(ChannelMember::User))

                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-channel_member_channel-channel_id")
                            .from(ChannelMember::Table, ChannelMember::Channel)
                            .to(Channel::Table, Channel::Id)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-channel_member_user-user_id")
                            .from(ChannelMember::Table, ChannelMember::User)
                            .to(User::Table, User::Id)
                    )

                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ChannelMember::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum ChannelMember {
    Table,
    Channel,
    User,
}
