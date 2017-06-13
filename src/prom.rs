use prometheus::{Registry, TextEncoder, Encoder};
use std::env;
use std::error::Error;

pub fn get_encoded_from_registry(registry: Registry) -> String {
    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    let metric_familys = registry.gather();

    return match encoder.encode(&metric_familys, &mut buffer).err() {
               Some(e) => e.description().to_owned(),
               _ => String::from_utf8(buffer).unwrap_or("".to_owned()),
           };
}

pub fn prometheus_prefix() -> String {
    env::var("PROMETHEUS_PREFIX").unwrap_or(String::from("redis"))
}
