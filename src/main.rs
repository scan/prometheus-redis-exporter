#[macro_use] extern crate nickel;
extern crate redis;
extern crate prometheus;

use nickel::{Nickel, MediaType, HttpRouter};
use redis::{InfoDict};
use prometheus::{Opts, Registry, Gauge, TextEncoder, Encoder};
use std::env;

static GAUGES: [(&'static str, &'static str, &'static str); 7] = [
  ("connected_slaves", "connected_replicas", "Number of connected replicas"),
  ("total_commands_processed", "total_commands_processed", "Total commands that reated this server"),
  ("used_cpu_sys", "used_cpu_sys_seconds", "Seconds of system CPU time used"),
  ("used_cpu_user", "used_cpu_user_seconds", "Seconds of user CPU time used"),
  ("used_memory_peak", "used_memory_peak_bytes", "Used memory peak"),
  ("used_memory_rss", "used_memory_rss_bytes", "Used resident memory by process"),
  ("used_memory", "used_memory_bytes", "Used memory by process"),
];

fn get_encoded_from_registry(registry: Registry) -> String {
    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    let metric_familys = registry.gather();
    encoder.encode(&metric_familys, &mut buffer).unwrap();

    String::from_utf8(buffer).unwrap()
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

fn fetch_redis_info() -> InfoDict {
    let client = redis::Client::open(get_redis_url().as_str()).unwrap();
    let con = client.get_connection().unwrap();

    redis::cmd("INFO").query(&con).unwrap()
}

fn main() {
    let mut server = Nickel::new();
    let prometheus_prefix: String = env::var("PROMETHEUS_PREFIX").unwrap_or(String::from("redis"));

    server.get("/metrics", middleware! { |_, mut res|
        let registry = Registry::new();
        let info = fetch_redis_info();

        for (_, &(redis_name, gauge_name, gauge_desc)) in GAUGES.iter().enumerate() {
            if !info.contains_key(&redis_name) {
                continue;
            }

            let opts = Opts::new((prometheus_prefix.clone() + "_" + gauge_name), gauge_desc.to_owned());
            let gauge = Gauge::with_opts(opts).unwrap();
            let val: String = info.get(redis_name).unwrap_or(String::new());

            gauge.set(val.parse().unwrap());

            registry.register(Box::new(gauge.clone())).unwrap();
        }

        res.set(MediaType::Txt);

        get_encoded_from_registry(registry)
    });

    let app_port: String = env::var("APP_PORT").unwrap_or(String::from("9121"));
    let _ = server.listen(format!("0.0.0.0:{}", app_port));
}
