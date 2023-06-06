pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_user;
mod m20230316_064242_create_session;
mod m20230527_132824_add_accent_color_to_user;
mod m20230527_183738_change_flags_to_u64;
mod m20230529_033401_add_more_session_details;
mod m20230604_022146_create_relationships;
mod m20230604_223625_create_channel;
mod m20230604_231009_create_message;
mod m20230604_235432_fk_channel_last_message_id_to_message_id;
mod m20230605_023233_create_channel_member;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_user::Migration),
            Box::new(m20230316_064242_create_session::Migration),
            Box::new(m20230527_132824_add_accent_color_to_user::Migration),
            Box::new(m20230527_183738_change_flags_to_u64::Migration),
            Box::new(m20230529_033401_add_more_session_details::Migration),
            Box::new(m20230604_022146_create_relationships::Migration),
            Box::new(m20230604_223625_create_channel::Migration),
            Box::new(m20230604_231009_create_message::Migration),
            Box::new(m20230604_235432_fk_channel_last_message_id_to_message_id::Migration),
            Box::new(m20230605_023233_create_channel_member::Migration),
        ]
    }
}
