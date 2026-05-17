// Requires a live GemStone/S stone for GET /health/gemstone.
//
// Start:
//
// cargo run -- --host 127.0.0.1 --port 3000 --workers 2
//
// Then in another shell:
//
// curl -i http://127.0.0.1:3000/
// curl -i http://127.0.0.1:3000/health/local
// curl -i http://127.0.0.1:3000/health/gemstone

use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue},
    middleware::{self, Next},
    response::Response,
};
use std::{env, error::Error};

const EXAMPLE_MIDDLEWARE_HEADER: &str = "x-gemstone-rs-example-middleware";
const SERVICE_HEADER: &str = "x-gemstone-rs-service";
const SERVICE_VERSION_HEADER: &str = "x-gemstone-rs-service-version";
const CACHE_CONTROL_HEADER: &str = "cache-control";
const CONTENT_TYPE_OPTIONS_HEADER: &str = "x-content-type-options";

type AppError = Box<dyn Error + Send + Sync>;
type AppResult<T> = Result<T, AppError>;

#[tokio::main]
async fn main() -> AppResult<()> {
    let options = Options::parse(env::args().skip(1))?;
    if options.routes {
        print_routes(&options);
        return Ok(());
    }

    let health = gemstone_rs_axum::health_pool_from_env(options.workers);
    if let Some(message) = health.unavailable_message() {
        eprintln!("GemStone health unavailable at startup: {message}");
    }
    let listener = tokio::net::TcpListener::bind(options.addr()).await?;
    println!(
        "gemstone-rs Axum service running at http://{}/",
        listener.local_addr()?
    );
    for route in gemstone_rs_axum::ROUTES {
        println!("{route}");
    }
    axum::serve(
        listener,
        gemstone_rs_axum::router_with_health_pool(health, "gemstone-rs Axum service example")
            .layer(middleware::from_fn(example_middleware)),
    )
    .await?;
    Ok(())
}

async fn example_middleware(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        HeaderName::from_static(EXAMPLE_MIDDLEWARE_HEADER),
        HeaderValue::from_static("axum"),
    );
    headers.insert(
        HeaderName::from_static(SERVICE_HEADER),
        HeaderValue::from_static("gemstone-rs-axum-service"),
    );
    headers.insert(
        HeaderName::from_static(SERVICE_VERSION_HEADER),
        HeaderValue::from_static(env!("CARGO_PKG_VERSION")),
    );
    headers.insert(
        HeaderName::from_static(CACHE_CONTROL_HEADER),
        HeaderValue::from_static("no-store"),
    );
    headers.insert(
        HeaderName::from_static(CONTENT_TYPE_OPTIONS_HEADER),
        HeaderValue::from_static("nosniff"),
    );
    response
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
    for route in gemstone_rs_axum::ROUTES {
        println!("  {route}");
    }
    println!();
    println!("Start:");
    println!(
        "  cargo run -- --host {} --port {} --workers {}",
        options.host, options.port, options.workers
    );
}

fn print_usage() {
    println!(
        "usage: cargo run -- [--host <host>] [--port <port>] [--workers <count>] [--routes]"
    );
}
