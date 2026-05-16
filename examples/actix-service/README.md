# Actix Service Example

This is a checked Rust-native Actix Web service for teams that want
`gemstone-rs` inside an async web application. It uses the reusable
`gemstone-rs-actix` adapter crate so application code does not copy
health-route handlers.

It exposes the same minimal contract used by the Python FastAPI/Litestar
examples:

```text
GET /
GET /health/local
GET /health/gemstone
```

Check the route map without starting a server:

```bash
cargo run --manifest-path examples/actix-service/Cargo.toml -- --routes
```

Run the local service:

```bash
cargo run --manifest-path examples/actix-service/Cargo.toml -- --host 127.0.0.1 --port 3000 --workers 2
```

Then in another shell:

```bash
curl -i http://127.0.0.1:3000/
curl -i http://127.0.0.1:3000/health/local
curl -i http://127.0.0.1:3000/health/gemstone
```

The service starts through `gemstone_rs_actix::health_pool_from_env`, then
passes that health backend to `gemstone_rs_actix::scope_with_health_pool`. That
lets the server boot before credentials are configured: `/` and `/health/local`
respond, while `/health/gemstone` returns a `503` JSON error until the pool is
available. When the pool is ready, the adapter wraps the shared
`gemstone_rs::web` response helper inside `actix_web::web::block`; each
underlying GemStone `Session` stays pinned to one worker thread.

Run the route smoke check:

```bash
python3 scripts/framework_route_smoke.py
```

Responses include `x-gemstone-rs-adapter: actix`, an `x-gemstone-rs-route`
value of `root`, `health.local`, or `health.gemstone`, and request trace
headers: `x-gemstone-rs-request-id`, `x-gemstone-rs-request-method`, and
`x-gemstone-rs-request-path`. Send `x-request-id` in a request when you want a
proxy, smoke test, or log stream to correlate the response.

Use the installed scaffold when you want to start a separate application:

```bash
gemstone-rs examples scaffold actix_service ./gemstone-rs-actix-service
```

The dependency-free source-checkout service remains useful for checking the same
route contract without Axum, Actix, Tokio, or serde:

```bash
cargo run -p gemstone-rs --example http_service -- --routes
cargo run -p gemstone-rs --example http_service -- --port 3000
```
