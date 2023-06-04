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
                    .table(Relationship::Table)
                    .if_not_exists()

                    .col(ColumnDef::new(Relationship::Creator).big_integer().not_null())
                    .col(ColumnDef::new(Relationship::Peer).big_integer().not_null())
                    .col(ColumnDef::new(Relationship::RelationshipType).integer().not_null())
                    .col(ColumnDef::new(Relationship::Timestamp).date_time().not_null())

                    .primary_key(Index::create().col(Relationship::Creator).col(Relationship::Peer))

                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-relationship_creator-user_id")
                            .from(Relationship::Table, Relationship::Creator)
                            .to(User::Table, User::Id)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-relationship_peer-user_id")
                            .from(Relationship::Table, Relationship::Peer)
                            .to(User::Table, User::Id)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Relationship::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Relationship {
    Table,
    Creator,
    Peer,
    RelationshipType,
    Timestamp
}
