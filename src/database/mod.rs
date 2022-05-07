mod models;

use rocket_sync_db_pools::{database};

#[database("epl_db")]
pub struct EplDb(rocket_sync_db_pools::diesel::PgConnection);