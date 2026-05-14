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

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use gemstone_rs::{Config, Session, Value};
use serde_json::json;
use std::{env, error::Error};

type AppError = Box<dyn Error + Send + Sync>;
type AppResult<T> = Result<T, AppError>;

#[actix_web::main]
async fn main() -> AppResult<()> {
    let options = Options::parse(env::args().skip(1))?;
    if options.routes {
        print_routes(&options);
        return Ok(());
    }

    let server = HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(root))
            .route("/health/local", web::get().to(health_local))
            .route("/health/gemstone", web::get().to(health_gemstone))
    })
    .bind(options.addr())?;

    let addrs = server.addrs();
    let display_addr = addrs
        .first()
        .map(std::string::ToString::to_string)
        .unwrap_or_else(|| options.addr());
    println!("gemstone-rs Actix service running at http://{display_addr}/");
    println!("GET /");
    println!("GET /health/local");
    println!("GET /health/gemstone");
    server.run().await?;
    Ok(())
}

async fn root() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "name": "gemstone-rs Actix service example",
        "endpoints": {
            "local": "/health/local",
            "gemstone": "/health/gemstone"
        }
    }))
}

async fn health_local() -> impl Responder {
    HttpResponse::Ok().json(json!({"ok": true}))
}

async fn health_gemstone() -> impl Responder {
    match web::block(gemstone_health_value).await {
        Ok(Ok(value)) => HttpResponse::Ok().json(json!({"result": value})),
        Ok(Err(err)) => HttpResponse::InternalServerError().json(json!({"error": err.to_string()})),
        Err(err) => HttpResponse::InternalServerError().json(json!({"error": err.to_string()})),
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
    println!("gemstone-rs Actix service example");
    println!("  bind: {}", options.addr());
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
