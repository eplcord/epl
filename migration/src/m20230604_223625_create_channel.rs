use crate::m20220101_000001_create_user::User;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Channel::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Channel::Id)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Channel::Type).integer().not_null())
                    // TODO: Since we don't have guilds yet at this point, we can make a foreign key later
                    .col(ColumnDef::new(Channel::GuildID).big_integer())
                    .col(ColumnDef::new(Channel::Position).integer())
                    .col(ColumnDef::new(Channel::Name).string())
                    .col(ColumnDef::new(Channel::Topic).string())
                    .col(ColumnDef::new(Channel::Nsfw).boolean())
                    // TODO: This will be foreign keyed to a message when that is implemented
                    .col(ColumnDef::new(Channel::LastMessageID).big_integer())
                    .col(ColumnDef::new(Channel::Bitrate).integer())
                    .col(ColumnDef::new(Channel::UserLimit).integer())
                    .col(ColumnDef::new(Channel::RateLimitPerUser).integer())
                    // TODO: To be foreign keyed once we implement files
                    .col(ColumnDef::new(Channel::Icon).string())
                    .col(ColumnDef::new(Channel::OwnerID).big_integer())
                    .col(ColumnDef::new(Channel::ApplicationID).big_integer())
                    .col(ColumnDef::new(Channel::Managed).boolean())
                    .col(ColumnDef::new(Channel::ParentID).big_integer())
                    .col(ColumnDef::new(Channel::LastPinTimestamp).date_time())
                    // TODO: Foreign keyed once we have voice regions
                    .col(ColumnDef::new(Channel::RTCRegion).string())
                    .col(ColumnDef::new(Channel::VideoQualityMode).integer())
                    .col(ColumnDef::new(Channel::DefaultAutoArchiveDuration).integer())
                    .col(ColumnDef::new(Channel::Permissions).string())
                    .col(ColumnDef::new(Channel::Flags).big_integer())
                    .col(ColumnDef::new(Channel::DefaultThreadRateLimitPerUser).integer())
                    .col(ColumnDef::new(Channel::DefaultSortOrder).integer())
                    .col(ColumnDef::new(Channel::DefaultForumLayout).integer())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-channel_owner_id-user_id")
                            .from(Channel::Table, Channel::OwnerID)
                            .to(User::Table, User::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-relationship_peer-user_id")
                            .from(Channel::Table, Channel::ParentID)
                            .to(Channel::Table, Channel::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Channel::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum Channel {
    Table,
    Id,
    Type,
    GuildID,
    Position,
    Name,
    Topic,
    Nsfw,
    LastMessageID,
    Bitrate,
    UserLimit,
    RateLimitPerUser,
    Icon,
    OwnerID,
    ApplicationID,
    Managed,
    ParentID,
    LastPinTimestamp,
    RTCRegion,
    VideoQualityMode,
    DefaultAutoArchiveDuration,
    Permissions,
    Flags,
    DefaultThreadRateLimitPerUser,
    DefaultSortOrder,
    DefaultForumLayout,
}
