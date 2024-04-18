use sea_orm_migration::prelude::*;
use crate::ColumnType::{BigInteger, String};
use crate::m20220101_000001_create_user::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Frecency::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Frecency::User).big_integer().not_null())
                    .col(ColumnDef::new(Frecency::DataVersion).big_integer().not_null().default(0))
                    .col(ColumnDef::new(Frecency::FavoriteGifs).json_binary())
                    .col(ColumnDef::new(Frecency::FavoriteGifsHideTooltip).boolean().not_null().default(false))
                    .col(ColumnDef::new(Frecency::FavoriteStickers).array(BigInteger))
                    .col(ColumnDef::new(Frecency::StickerFrecency).json_binary())
                    .col(ColumnDef::new(Frecency::FavoriteEmojis).array(String(StringLen::None)))
                    .col(ColumnDef::new(Frecency::EmojiFrecency).json_binary())
                    .col(ColumnDef::new(Frecency::ApplicationCommandFrecency).json_binary())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_frecency-user_user-id")
                            .from(Frecency::Table, Frecency::User)
                            .to(User::Table, User::Id)
                    )
                    .primary_key(
                        Index::create()
                            .col(Frecency::User)
                            .col(Frecency::DataVersion)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Frecency::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Frecency {
    Table,
    User,
    DataVersion,
    FavoriteGifs,
    FavoriteGifsHideTooltip,
    FavoriteStickers,
    StickerFrecency,
    FavoriteEmojis,
    EmojiFrecency,
    ApplicationCommandFrecency
}
