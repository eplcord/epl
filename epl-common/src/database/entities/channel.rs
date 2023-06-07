//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.3

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "channel")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub r#type: i32,
    pub guild_id: Option<i64>,
    pub position: Option<i32>,
    pub name: Option<String>,
    pub topic: Option<String>,
    pub nsfw: Option<bool>,
    pub last_message_id: Option<i64>,
    pub bitrate: Option<i32>,
    pub user_limit: Option<i32>,
    pub rate_limit_per_user: Option<i32>,
    pub icon: Option<String>,
    pub owner_id: Option<i64>,
    pub application_id: Option<i64>,
    pub managed: Option<bool>,
    pub parent_id: Option<i64>,
    pub last_pin_timestamp: Option<DateTime>,
    pub rtc_region: Option<String>,
    pub video_quality_mode: Option<i32>,
    pub default_auto_archive_duration: Option<i32>,
    pub permissions: Option<String>,
    pub flags: Option<i64>,
    pub default_thread_rate_limit_per_user: Option<i32>,
    pub default_sort_order: Option<i32>,
    pub default_forum_layout: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::ParentId",
        to = "Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    SelfRef,
    #[sea_orm(
        belongs_to = "super::message::Entity",
        from = "Column::LastMessageId",
        to = "super::message::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Message,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::OwnerId",
        to = "super::user::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    User,
}

impl Related<super::message::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Message.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        super::channel_member::Relation::User.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::channel_member::Relation::Channel.def().rev())
    }
}

// impl Related<super::user::Entity> for Entity {
//     fn to() -> RelationDef {
//         super::relationship::Relation::User.def()
//     }
//     fn via() -> Option<RelationDef> {
//         Some(super::relationship::Relation::Channel.def().rev())
//     }
// }

impl ActiveModelBehavior for ActiveModel {}
