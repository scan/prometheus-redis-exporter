extern crate iron;
extern crate redis;
extern crate prometheus;
extern crate time;

mod redis_client;
mod prom;
mod metrics;

use iron::prelude::*;
use std::env;

use metrics::metrics_handler;

fn main() {
    let app_port: String = env::var("APP_PORT").unwrap_or("9121".to_owned());
    let chain = Chain::new(metrics_handler);

    Iron::new(chain).http(format!("0.0.0.0:{}", app_port)).unwrap();
}
