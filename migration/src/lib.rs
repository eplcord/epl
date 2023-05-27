pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_user;
mod m20230316_064242_create_session;
mod m20230527_132824_add_accent_color_to_user;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_user::Migration),
            Box::new(m20230316_064242_create_session::Migration),
            Box::new(m20230527_132824_add_accent_color_to_user::Migration),
        ]
    }
}
