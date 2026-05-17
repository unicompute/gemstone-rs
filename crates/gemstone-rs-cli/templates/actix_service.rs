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

use actix_web::{middleware::DefaultHeaders, App, HttpServer};
use std::{env, error::Error};

type AppError = Box<dyn Error + Send + Sync>;
type AppResult<T> = Result<T, AppError>;

const EXAMPLE_MIDDLEWARE_HEADER: &str = "x-gemstone-rs-example-middleware";
const SERVICE_HEADER: &str = "x-gemstone-rs-service";
const SERVICE_VERSION_HEADER: &str = "x-gemstone-rs-service-version";
const CACHE_CONTROL_HEADER: &str = "cache-control";
const CONTENT_TYPE_OPTIONS_HEADER: &str = "x-content-type-options";

#[actix_web::main]
async fn main() -> AppResult<()> {
    let options = Options::parse(env::args().skip(1))?;
    if options.routes {
        print_routes(&options);
        return Ok(());
    }

    let health = gemstone_rs_actix::health_pool_from_env(options.workers);
    if let Some(message) = health.unavailable_message() {
        eprintln!("GemStone health unavailable at startup: {message}");
    }
    let server = HttpServer::new(move || {
        let health = health.clone();
        App::new()
            .wrap(
                DefaultHeaders::new()
                    .add((EXAMPLE_MIDDLEWARE_HEADER, "actix"))
                    .add((SERVICE_HEADER, "gemstone-rs-actix-service"))
                    .add((SERVICE_VERSION_HEADER, env!("CARGO_PKG_VERSION")))
                    .add((CACHE_CONTROL_HEADER, "no-store"))
                    .add((CONTENT_TYPE_OPTIONS_HEADER, "nosniff")),
            )
            .service(gemstone_rs_actix::scope_with_health_pool(
                health,
                "gemstone-rs Actix service example",
            ))
    })
    .bind(options.addr())?;

    let addrs = server.addrs();
    let display_addr = addrs
        .first()
        .map(std::string::ToString::to_string)
        .unwrap_or_else(|| options.addr());
    println!("gemstone-rs Actix service running at http://{display_addr}/");
    for route in gemstone_rs_actix::ROUTES {
        println!("{route}");
    }
    server.run().await?;
    Ok(())
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
    println!("gemstone-rs Actix service example");
    println!("  bind: {}", options.addr());
    println!("  workers: {}", options.workers);
    for route in gemstone_rs_actix::ROUTES {
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
