FROM scratch
ADD target/x86_64-unknown-linux-musl/release/prometheus-redis-exporter /prometheus-redis-exporter
EXPOSE 9121
ENTRYPOINT ["/prometheus-redis-exporter"]
