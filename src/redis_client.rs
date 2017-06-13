use redis::{self, InfoDict, RedisError};
use std::env;

pub fn get_redis_url() -> String {
    let redis_host: String = env::var("REDIS_HOST").unwrap_or(String::from("localhost"));
    let redis_port: String = env::var("REDIS_PORT").unwrap_or(String::from("6379"));
    let redis_password: String = env::var("REDIS_PASSWORD").unwrap_or(String::new());

    if redis_password.is_empty() {
        return format!("redis://{}:{}/", redis_host, redis_port);
    } else {
        return format!("redis://:{}@{}:{}/", redis_password, redis_host, redis_port);
    }
}

pub fn fetch_redis_info() -> Result<InfoDict, RedisError> {
    let client = redis::Client::open(get_redis_url().as_str())?;
    let con = client.get_connection()?;

    redis::cmd("INFO").query(&con)
}
