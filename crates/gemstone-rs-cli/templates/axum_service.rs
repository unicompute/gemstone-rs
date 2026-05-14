// Requires a live GemStone/S stone for GET /health/gemstone.
//
// Start:
//
// cargo run -- --host 127.0.0.1 --port 3000
//
// Then in another shell:
//
// curl -i http://127.0.0.1:3000/
// curl -i http://127.0.0.1:3000/health/local
// curl -i http://127.0.0.1:3000/health/gemstone

use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use gemstone_rs::{Config, Session, Value};
use serde_json::json;
use std::{env, error::Error, net::SocketAddr};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let options = Options::parse(env::args().skip(1))?;
    if options.routes {
        print_routes(&options);
        return Ok(());
    }

    let app = Router::new()
        .route("/", get(root))
        .route("/health/local", get(health_local))
        .route("/health/gemstone", get(health_gemstone));

    let addr: SocketAddr = format!("{}:{}", options.host, options.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!(
        "gemstone-rs Axum service running at http://{}/",
        listener.local_addr()?
    );
    println!("GET /");
    println!("GET /health/local");
    println!("GET /health/gemstone");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn root() -> impl IntoResponse {
    Json(json!({
        "name": "gemstone-rs Axum service example",
        "endpoints": {
            "local": "/health/local",
            "gemstone": "/health/gemstone"
        }
    }))
}

async fn health_local() -> impl IntoResponse {
    Json(json!({"ok": true}))
}

async fn health_gemstone() -> impl IntoResponse {
    match gemstone_health_value() {
        Ok(value) => (StatusCode::OK, Json(json!({"result": value}))),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": err.to_string()})),
        ),
    }
}

fn gemstone_health_value() -> Result<i64, Box<dyn Error>> {
    let mut session = Session::login(Config::from_env()?)?;
    let value = session.eval("3 + 4")?;
    let Value::SmallInt(value) = value else {
        return Err("GemStone health check returned a non-SmallInt value".into());
    };
    session.logout()?;
    Ok(value)
}

#[derive(Debug, Eq, PartialEq)]
struct Options {
    host: String,
    port: u16,
    routes: bool,
}

impl Options {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, Box<dyn Error>> {
        let mut host = "127.0.0.1".to_string();
        let mut port = 3000;
        let mut routes = false;
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--host" => {
                    host = args.next().ok_or("missing value after --host")?;
                }
                "--port" => {
                    port = args.next().ok_or("missing value after --port")?.parse()?;
                }
                "--routes" => routes = true,
                "-h" | "--help" => {
                    print_usage();
                    std::process::exit(0);
                }
                other => return Err(format!("unknown option: {other}").into()),
            }
        }

        Ok(Self { host, port, routes })
    }
}

fn print_routes(options: &Options) {
    println!("gemstone-rs Axum service example");
    println!("  bind: {}:{}", options.host, options.port);
    println!("  GET /");
    println!("  GET /health/local");
    println!("  GET /health/gemstone");
    println!();
    println!("Start:");
    println!(
        "  cargo run -- --host {} --port {}",
        options.host, options.port
    );
}

fn print_usage() {
    println!("usage: cargo run -- [--host <host>] [--port <port>] [--routes]");
}
