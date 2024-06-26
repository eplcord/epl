use std::env;

// TODO: Make this better lol
#[derive(Clone)]
pub struct EplOptions {
    pub name: String,
    pub description: String,
    pub url: String,
    pub gateway_url: String,
    pub mediaproxy_url: Option<String>,
    pub listen_addr: String,
    pub database_url: String,
    pub nats_addr: String,
    pub lvsp_secret: String,
    pub require_ssl: bool,
    pub registration: bool,
    pub maxminddb: String,
    pub s3_bucket: String,
    pub pomelo: bool,
    pub tenor_key: Option<String>,
    pub cdn_url: String,
}
pub trait Options {
    fn get() -> EplOptions;
}
impl Options for EplOptions {
    fn get() -> EplOptions {
        EplOptions {
            name: env::var("NAME").unwrap_or_else(|_| "Epl".to_string()),
            description: env::var("DESCRIPTION").unwrap_or_else(|_| "An Epl instance".to_string()),
            url: env::var("URL").expect("URL is required!"),
            gateway_url: env::var("GATEWAY_URL").expect("GATEWAY_URL is required!"),
            mediaproxy_url: env::var("MEDIAPROXY_URL").ok(),
            listen_addr: env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:3926".to_string()),
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://epl:epl@127.0.0.1/epl".to_string()),
            nats_addr: env::var("NATS_ADDR").unwrap_or_else(|_| "127.0.0.1:4222".to_string()),
            lvsp_secret: env::var("LVSP_SECRET").expect("LVSP_SECRET is required!"),
            require_ssl: env::var("REQUIRE_SSL")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap(),
            registration: env::var("REGISTRATION")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap(),
            maxminddb: env::var("MAXMIND_DB_PATH").unwrap_or_else(|_| "GeoLite2-City.mmdb".to_string()),
            s3_bucket: env::var("S3_BUCKET").unwrap_or_else(|_| "epl".to_string()),
            pomelo: env::var("POMELO")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap(),
            tenor_key: env::var("TENOR_KEY").ok(),
            cdn_url: env::var("CDN_URL").expect("CDN_URL is required!"),
        }
    }
}
