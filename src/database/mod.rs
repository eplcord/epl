mod models;

use rocket_sync_db_pools::{diesel, database};

#[database("epl_db")]
pub struct EplDb(diesel::PgConnection);