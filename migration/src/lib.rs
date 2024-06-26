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
mod m20230607_054224_rename_message_reference_in_message;
mod m20230607_055518_add_reference_channel_id_to_message;
mod m20231118_011903_create_mentions;
mod m20240319_061446_create_notes;
mod m20240321_124842_pomelo;
mod m20240327_195051_create_pins;
mod m20240331_060215_create_embeds;
mod m20240402_025357_create_april_fools_2024;
mod m20240412_003408_create_files;
mod m20240412_062553_create_message_attachments;
mod m20240413_215635_add_requested_deletion_to_files;
mod m20240414_025415_create_reactions;
mod m20240417_080833_create_settings;
mod m20240418_040054_create_frecency;

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
            Box::new(m20230607_054224_rename_message_reference_in_message::Migration),
            Box::new(m20230607_055518_add_reference_channel_id_to_message::Migration),
            Box::new(m20231118_011903_create_mentions::Migration),
            Box::new(m20240319_061446_create_notes::Migration),
            Box::new(m20240321_124842_pomelo::Migration),
            Box::new(m20240327_195051_create_pins::Migration),
            Box::new(m20240331_060215_create_embeds::Migration),
            Box::new(m20240402_025357_create_april_fools_2024::Migration),
            Box::new(m20240412_003408_create_files::Migration),
            Box::new(m20240412_062553_create_message_attachments::Migration),
            Box::new(m20240413_215635_add_requested_deletion_to_files::Migration),
            Box::new(m20240414_025415_create_reactions::Migration),
            Box::new(m20240417_080833_create_settings::Migration),
            Box::new(m20240418_040054_create_frecency::Migration),
        ]
    }
}
