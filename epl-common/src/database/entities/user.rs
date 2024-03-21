//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.1

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub system: bool,
    pub bot: bool,
    #[sea_orm(column_type = "Text")]
    pub username: String,
    #[sea_orm(column_type = "Text")]
    pub password_hash: String,
    pub discriminator: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub bio: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub pronouns: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub avatar: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub avatar_decoration: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub banner: Option<String>,
    pub banner_colour: Option<String>,
    pub date_of_birth: Option<DateTime>,
    #[sea_orm(column_type = "Text")]
    pub email: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub phone: Option<String>,
    pub mfa_enabled: bool,
    pub acct_verified: bool,
    pub flags: i64,
    pub nsfw_allowed: bool,
    pub purchased_flags: Option<i32>,
    pub premium_flags: Option<i32>,
    pub premium_type: Option<i32>,
    pub premium_since: Option<DateTime>,
    #[sea_orm(column_type = "Text", nullable)]
    pub accent_color: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub display_name: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub legacy_name: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::channel::Entity")]
    Channel,
    #[sea_orm(has_many = "super::message::Entity")]
    Message,
}

impl Related<super::channel::Entity> for Entity {
    fn to() -> RelationDef {
        super::channel_member::Relation::Channel.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::channel_member::Relation::User.def().rev())
    }
}

impl Related<super::message::Entity> for Entity {
    fn to() -> RelationDef {
        super::mention::Relation::Message.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::mention::Relation::User.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
