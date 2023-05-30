use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::ConnectionTrait;
use crate::m20230316_064242_create_session::Session;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Truncate database because we don't have a session_id for sessions
        let db = manager.get_connection();

        db.execute_unprepared(
            "TRUNCATE session;"
        ).await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Session::Table)
                    .drop_column(Session::Token)
                    .add_column(
                        ColumnDef::new(Alias::new("session_id"))
                            .text()
                            .not_null()
                            .primary_key()
                    )
                    .add_column(
                        ColumnDef::new(Session::Token)
                            .text()
                            .not_null()
                    )
                    .add_column(
                        ColumnDef::new(Alias::new("status"))
                            .text()
                            .not_null()
                    )
                    .add_column(
                        ColumnDef::new(Alias::new("os"))
                        .text()
                    )
                    .add_column(
                        ColumnDef::new(Alias::new("platform"))
                        .text()
                    )
                    .add_column(
                        ColumnDef::new(Alias::new("last_used"))
                        .date_time()
                        .not_null()
                    )
                    .add_column(
                        ColumnDef::new(Alias::new("location"))
                        .text()
                    )
                    .to_owned()
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Session::Table)
                    .drop_column(Session::Token)
                    .add_column(
                        ColumnDef::new(Session::Token)
                            .text()
                            .not_null()
                            .primary_key()
                    )
                    .drop_column(Alias::new("session_id"))
                    .drop_column(Alias::new("status"))
                    .drop_column(Alias::new("os"))
                    .drop_column(Alias::new("platform"))
                    .drop_column(Alias::new("last_used"))
                    .drop_column(Alias::new("location"))
                    .to_owned()
            ).await
    }
}