#[macro_use]
extern crate nickel;
extern crate redis;
extern crate prometheus;
extern crate time;

use nickel::{Nickel, MediaType, HttpRouter, NickelError};
use nickel::status::StatusCode;
use redis::{InfoDict, RedisError};
use prometheus::{Opts, Registry, Gauge, TextEncoder, Encoder};
use std::env;
use std::error::Error;

static GAUGES: [(&'static str, &'static str, &'static str); 7] =
    [("connected_slaves", "connected_replicas", "Number of connected replicas"),
     ("total_commands_processed",
      "total_commands_processed",
      "Total commands that reated this server"),
     ("used_cpu_sys", "used_cpu_sys_seconds", "Seconds of system CPU time used"),
     ("used_cpu_user", "used_cpu_user_seconds", "Seconds of user CPU time used"),
     ("used_memory_peak", "used_memory_peak_bytes", "Used memory peak"),
     ("used_memory_rss", "used_memory_rss_bytes", "Used resident memory by process"),
     ("used_memory", "used_memory_bytes", "Used memory by process")];

fn get_encoded_from_registry(registry: Registry) -> String {
    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    let metric_familys = registry.gather();

    return match encoder.encode(&metric_familys, &mut buffer).err() {
               Some(e) => e.description().to_owned(),
               _ => String::from_utf8(buffer).unwrap_or("".to_owned()),
           };
}

fn get_redis_url() -> String {
    let redis_host: String = env::var("REDIS_HOST").unwrap_or(String::from("localhost"));
    let redis_port: String = env::var("REDIS_PORT").unwrap_or(String::from("6379"));
    let redis_password: String = env::var("REDIS_PASSWORD").unwrap_or(String::new());

    if redis_password.is_empty() {
        return format!("redis://{}:{}/", redis_host, redis_port);
    } else {
        return format!("redis://:{}@{}:{}/", redis_password, redis_host, redis_port);
    }
}

fn fetch_redis_info() -> Result<InfoDict, RedisError> {
    let client = redis::Client::open(get_redis_url().as_str())?;
    let con = client.get_connection()?;

    redis::cmd("INFO").query(&con)
}

fn main() {
    let mut server = Nickel::new();
    let prometheus_prefix: String = env::var("PROMETHEUS_PREFIX").unwrap_or(String::from("redis"));

    server.get("/metrics",
               middleware! { |_, mut res|
        let start = time::now_utc();
        let registry = Registry::new();
        let info = match fetch_redis_info() {
            Err(e) => return Err(NickelError::new(res,
                e.description().to_owned(),
                StatusCode::InternalServerError)), // TODO: Log
            Ok(i) => i,
        };

        for (_, &(redis_name, gauge_name, gauge_desc)) in GAUGES.iter().enumerate() {
            if !info.contains_key(&redis_name) {
                continue;
            }

            let opts = Opts::new(
                (prometheus_prefix.clone() + "_" + gauge_name),
                gauge_desc.to_owned()
            );
            let gauge = match Gauge::with_opts(opts) {
                Ok(g) => g,
                Err(_) => continue,
            };
            let val: String = info.get(redis_name).unwrap_or(String::new());

            gauge.set(val.parse().unwrap_or(0.0)); // Owl

            let _ = registry.register(Box::new(gauge.clone()));
        }

        res.set(MediaType::Txt);

        let result = get_encoded_from_registry(registry);

        let duration = time::now_utc() - start;

        println!("Metrics rendered, time taken: {} ms", duration.num_microseconds().map(|x| (x as f64) / 1000.0).unwrap_or(0.0));

        result
    });

    let app_port: String = env::var("APP_PORT").unwrap_or(String::from("9121"));
    let _ = server.listen(format!("0.0.0.0:{}", app_port));
}
