use iron::prelude::*;
use iron::status;

use prometheus::{Opts, Registry, Gauge};

use time;
use redis_client::*;
use prom::*;

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

pub fn metrics_handler(_: &mut Request) -> IronResult<Response> {
    let start = time::now_utc();
    let registry = Registry::new();
    let info = match fetch_redis_info() {
        Err(e) => return Err(IronError::new(e, status::InternalServerError)),
        Ok(i) => i,
    };

    for (_, &(redis_name, gauge_name, gauge_desc)) in GAUGES.iter().enumerate() {
        if !info.contains_key(&redis_name) {
            continue;
        }

        let opts = Opts::new(
            format!("{}_{}", prometheus_prefix(), gauge_name),
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

    let result = get_encoded_from_registry(registry);
    let duration = time::now_utc() - start;

    println!("Metrics rendered, time taken: {} ms", duration.num_microseconds().map(|x| (x as f64) / 1000.0).unwrap_or(0.0));

    Ok(Response::with((status::Ok, result)))
}
