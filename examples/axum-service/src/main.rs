use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use gemstone_rs::{web as gemstone_web, Config, SessionWorkerPool};
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

    let pool = SessionWorkerPool::start(Config::from_env()?, options.workers)?;
    let listener = tokio::net::TcpListener::bind(options.addr()).await?;
    println!(
        "gemstone-rs Axum service running at http://{}/",
        listener.local_addr()?
    );
    println!("GET /");
    println!("GET /health/local");
    println!("GET /health/gemstone");
    axum::serve(listener, app(pool)).await?;
    Ok(())
}

#[derive(Clone)]
struct AppState {
    pool: SessionWorkerPool,
}

fn app(pool: SessionWorkerPool) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/health/local", get(health_local))
        .route("/health/gemstone", get(health_gemstone))
        .with_state(AppState { pool })
}

async fn root() -> impl IntoResponse {
    json_response(gemstone_web::index_response(
        "gemstone-rs Axum service example",
    ))
}

async fn health_local() -> impl IntoResponse {
    json_response(gemstone_web::local_health_response())
}

async fn health_gemstone(State(state): State<AppState>) -> impl IntoResponse {
    let pool = state.pool.clone();
    let response = match tokio::task::spawn_blocking(move || {
        gemstone_web::gemstone_health_response(&pool)
    })
    .await
    {
        Ok(response) => response,
        Err(err) => gemstone_web::JsonResponse::error(500, err.to_string()),
    };
    json_response(response)
}

fn json_response(response: gemstone_web::JsonResponse) -> impl IntoResponse {
    let status = StatusCode::from_u16(response.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (status, [(header::CONTENT_TYPE, "application/json")], response.body)
}

#[derive(Debug, Eq, PartialEq)]
struct Options {
    host: String,
    port: u16,
    workers: usize,
    routes: bool,
}

impl Options {
    fn parse(args: impl IntoIterator<Item = String>) -> AppResult<Self> {
        let mut host = "127.0.0.1".to_string();
        let mut port = 3000;
        let mut workers = 2;
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
                "--workers" => {
                    workers = args.next().ok_or("missing value after --workers")?.parse()?;
                    if workers == 0 {
                        return Err("--workers must be greater than zero".into());
                    }
                }
                "--routes" => routes = true,
                "-h" | "--help" => {
                    print_usage();
                    std::process::exit(0);
                }
                other => return Err(format!("unknown option: {other}").into()),
            }
        }

        Ok(Self {
            host,
            port,
            workers,
            routes,
        })
    }

    fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

fn print_routes(options: &Options) {
    println!("gemstone-rs Axum service example");
    println!("  bind: {}", options.addr());
    println!("  workers: {}", options.workers);
    println!("  GET /");
    println!("  GET /health/local");
    println!("  GET /health/gemstone");
    println!();
    println!("Start:");
    println!(
        "  cargo run --manifest-path examples/axum-service/Cargo.toml -- --host {} --port {} --workers {}",
        options.host, options.port, options.workers
    );
}

fn print_usage() {
    println!(
        "usage: cargo run --manifest-path examples/axum-service/Cargo.toml -- [--host <host>] [--port <port>] [--workers <count>] [--routes]"
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
                workers: 2,
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
            "--workers".to_string(),
            "3".to_string(),
            "--routes".to_string(),
        ])
        .unwrap();
        assert_eq!(options.host, "127.0.0.2");
        assert_eq!(options.port, 3100);
        assert_eq!(options.workers, 3);
        assert!(options.routes);
        assert_eq!(options.addr(), "127.0.0.2:3100");
    }

    #[test]
    fn rejects_zero_workers() {
        let err = Options::parse(["--workers".to_string(), "0".to_string()]).unwrap_err();
        assert!(err.to_string().contains("greater than zero"));
    }

    #[test]
    fn rejects_unknown_option() {
        let err = Options::parse(["--bad".to_string()]).unwrap_err();
        assert!(err.to_string().contains("unknown option"));
    }
}
