use sea_orm_migration::prelude::*;
use crate::m20220101_000001_create_user::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Note::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Note::Creator)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Note::Subject)
                            .big_integer()
                            .not_null()
                    )
                    .col(
                        ColumnDef::new(Note::Text)
                            .string()
                            .not_null()
                    )
                    .primary_key(
                        Index::create()
                            .col(Note::Creator)
                            .col(Note::Subject)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_creator_user_id")
                            .from(Note::Table, Note::Creator)
                            .to(User::Table, User::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_subject_user_id")
                            .from(Note::Table, Note::Subject)
                            .to(User::Table, User::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Note::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Note {
    Table,
    Creator,
    Subject,
    Text,
}
