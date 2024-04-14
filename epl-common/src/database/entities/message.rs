//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "message")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub channel_id: i64,
    pub author: Option<i64>,
    pub content: String,
    pub timestamp: DateTime,
    pub edited_timestamp: Option<DateTime>,
    pub tts: bool,
    pub mention_everyone: bool,
    pub nonce: Option<String>,
    pub pinned: bool,
    pub webhook_id: Option<i64>,
    pub r#type: i32,
    pub application_id: Option<i64>,
    pub reference_message_id: Option<i64>,
    pub flags: Option<i32>,
    pub reference_channel_id: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::channel::Entity",
        from = "Column::ChannelId",
        to = "super::channel::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Channel2,
    #[sea_orm(
        belongs_to = "super::channel::Entity",
        from = "Column::ReferenceChannelId",
        to = "super::channel::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Channel1,
    #[sea_orm(has_many = "super::embed::Entity")]
    Embed,
    #[sea_orm(has_many = "super::mention::Entity")]
    Mention,
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::ReferenceMessageId",
        to = "Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    SelfRef,
    #[sea_orm(has_many = "super::message_attachment::Entity")]
    MessageAttachment,
    #[sea_orm(has_many = "super::pin::Entity")]
    Pin,
    #[sea_orm(has_many = "super::reaction::Entity")]
    Reaction,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::Author",
        to = "super::user::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    User,
}

impl Related<super::embed::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Embed.def()
    }
}

impl Related<super::mention::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Mention.def()
    }
}

impl Related<super::message_attachment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MessageAttachment.def()
    }
}

impl Related<super::pin::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Pin.def()
    }
}

impl Related<super::reaction::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Reaction.def()
    }
}

impl Related<super::channel::Entity> for Entity {
    fn to() -> RelationDef {
        super::pin::Relation::Channel.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::pin::Relation::Message.def().rev())
    }
}

impl Related<super::file::Entity> for Entity {
    fn to() -> RelationDef {
        super::message_attachment::Relation::File.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::message_attachment::Relation::Message.def().rev())
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        super::mention::Relation::User.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::mention::Relation::Message.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
