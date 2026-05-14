use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use gemstone_rs::{Config, Session, Value};
use serde_json::json;
use std::{env, error::Error};

type AppError = Box<dyn Error + Send + Sync>;
type AppResult<T> = Result<T, AppError>;

#[tokio::main]
async fn main() -> AppResult<()> {
    let options = Options::parse(env::args().skip(1))?;
    if options.routes {
        print_routes(&options);
        return Ok(());
    }

    let listener = tokio::net::TcpListener::bind(options.addr()).await?;
    println!(
        "gemstone-rs Axum service running at http://{}/",
        listener.local_addr()?
    );
    println!("GET /");
    println!("GET /health/local");
    println!("GET /health/gemstone");
    axum::serve(listener, app()).await?;
    Ok(())
}

fn app() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health/local", get(health_local))
        .route("/health/gemstone", get(health_gemstone))
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
    let result = tokio::task::spawn_blocking(gemstone_health_value).await;
    match result {
        Ok(Ok(value)) => (StatusCode::OK, Json(json!({"result": value}))),
        Ok(Err(err)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": err.to_string()})),
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": err.to_string()})),
        ),
    }
}

fn gemstone_health_value() -> AppResult<i64> {
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
    fn parse(args: impl IntoIterator<Item = String>) -> AppResult<Self> {
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

    fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

fn print_routes(options: &Options) {
    println!("gemstone-rs Axum service example");
    println!("  bind: {}", options.addr());
    println!("  GET /");
    println!("  GET /health/local");
    println!("  GET /health/gemstone");
    println!();
    println!("Start:");
    println!(
        "  cargo run --manifest-path examples/axum-service/Cargo.toml -- --host {} --port {}",
        options.host, options.port
    );
}

fn print_usage() {
    println!(
        "usage: cargo run --manifest-path examples/axum-service/Cargo.toml -- [--host <host>] [--port <port>] [--routes]"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_defaults() {
        let options = Options::parse([]).unwrap();
        assert_eq!(
            options,
            Options {
                host: "127.0.0.1".to_string(),
                port: 3000,
                routes: false
            }
        );
        assert_eq!(options.addr(), "127.0.0.1:3000");
    }

    #[test]
    fn parses_host_port_and_routes() {
        let options = Options::parse([
            "--host".to_string(),
            "127.0.0.2".to_string(),
            "--port".to_string(),
            "3100".to_string(),
            "--routes".to_string(),
        ])
        .unwrap();
        assert_eq!(options.host, "127.0.0.2");
        assert_eq!(options.port, 3100);
        assert!(options.routes);
        assert_eq!(options.addr(), "127.0.0.2:3100");
    }

    #[test]
    fn rejects_unknown_option() {
        let err = Options::parse(["--bad".to_string()]).unwrap_err();
        assert!(err.to_string().contains("unknown option"));
    }
}
