//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "frecency")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub data_version: i64,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub favorite_gifs: Option<Json>,
    pub favorite_gifs_hide_tooltip: bool,
    pub favorite_stickers: Option<Vec<i64>>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub sticker_frecency: Option<Json>,
    pub favorite_emojis: Option<Vec<String>>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub emoji_frecency: Option<Json>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub application_command_frecency: Option<Json>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::User",
        to = "super::user::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    User,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
