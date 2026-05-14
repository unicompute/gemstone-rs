# Axum Service Example

This is a checked Rust-native Axum service for teams that want `gemstone-rs`
inside an async web application. It is intentionally kept outside the main
workspace so the core crate stays dependency-light.

It exposes the same minimal contract used by the Python FastAPI/Litestar
examples:

```text
GET /
GET /health/local
GET /health/gemstone
```

Check the route map without starting a server:

```bash
cargo run --manifest-path examples/axum-service/Cargo.toml -- --routes
```

Run the local service:

```bash
cargo run --manifest-path examples/axum-service/Cargo.toml -- --host 127.0.0.1 --port 3000
```

Then in another shell:

```bash
curl -i http://127.0.0.1:3000/
curl -i http://127.0.0.1:3000/health/local
curl -i http://127.0.0.1:3000/health/gemstone
```

The GemStone health route opens a session inside `tokio::task::spawn_blocking`
and evaluates `3 + 4`. Keep that pattern unless you introduce a deliberate
session worker or session pool for your deployment. Treat each GemStone
`Session` as thread-local and blocking.

Use the installed scaffold when you want to start a separate application:

```bash
gemstone-rs examples scaffold axum_service ./gemstone-rs-axum-service
```

The dependency-free source-checkout service remains useful for checking the
same route contract without Axum/Tokio:

```bash
cargo run -p gemstone-rs --example http_service -- --routes
cargo run -p gemstone-rs --example http_service -- --port 3000
```
