use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(User::Id)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(User::System).boolean().not_null().default(false))
                    .col(ColumnDef::new(User::Bot).boolean().not_null().default(false))
                    .col(ColumnDef::new(User::Username).text().not_null())
                    .col(ColumnDef::new(User::PasswordHash).text().not_null())
                    .col(ColumnDef::new(User::Discriminator).string_len(4).not_null())
                    .col(ColumnDef::new(User::Bio).text())
                    .col(ColumnDef::new(User::Pronouns).text())
                    .col(ColumnDef::new(User::Avatar).text())
                    .col(ColumnDef::new(User::AvatarDecoration).text())
                    .col(ColumnDef::new(User::Banner).text())
                    .col(ColumnDef::new(User::BannerColour).string_len(7))
                    .col(ColumnDef::new(User::DateOfBirth).timestamp())
                    .col(ColumnDef::new(User::Email).text().not_null())
                    .col(ColumnDef::new(User::Phone).text())
                    .col(ColumnDef::new(User::MFAEnabled).boolean().not_null().default(false))
                    .col(ColumnDef::new(User::AcctVerified).boolean().not_null().default(false))
                    .col(ColumnDef::new(User::Flags).integer().not_null().default(0))
                    .col(ColumnDef::new(User::NSFWAllowed).boolean().not_null().default(false))
                    .col(ColumnDef::new(User::PurchasedFlags).integer())
                    .col(ColumnDef::new(User::PremiumFlags).integer())
                    .col(ColumnDef::new(User::PremiumType).integer())
                    .col(ColumnDef::new(User::PremiumSince).timestamp())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum User {
    Table,
    Id,
    System,
    Bot,
    Username,
    PasswordHash,
    Discriminator,
    Bio,
    Pronouns,
    Avatar,
    AvatarDecoration,
    Banner,
    BannerColour,
    DateOfBirth,
    Email,
    Phone,
    MFAEnabled,
    AcctVerified,
    Flags,
    NSFWAllowed,
    PurchasedFlags,
    PremiumSince,
    PremiumFlags,
    PremiumType
}
