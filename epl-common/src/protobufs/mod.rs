pub mod discord_protos {
    pub mod discord_users {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/discord_protos.discord_users.v1.rs"));
        }
    }
}