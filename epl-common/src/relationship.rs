use sea_orm::{Condition, DatabaseConnection, EntityTrait};
use crate::database::entities::prelude::Relationship;
use crate::database::entities::relationship;
use sea_orm::prelude::*;


pub async fn get_relationship(
    user_a: i64,
    user_b: i64,
    conn: &DatabaseConnection
) -> Option<relationship::Model> {
    Relationship::find()
        .filter(
            Condition::any()
                .add(relationship::Column::Creator.eq(user_a))
                .add(relationship::Column::Creator.eq(user_b)),
        )
        .filter(
            Condition::any()
                .add(relationship::Column::Peer.eq(user_a))
                .add(relationship::Column::Peer.eq(user_b)),
        )
        .one(conn)
        .await
        .expect("Failed to access database!")
}