// Requires a live GemStone/S stone for GET /health/gemstone.
//
// This scaffold intentionally uses only the Rust standard library. It proves
// the service shape without adding Axum, Actix, Tokio, or serde dependencies.
//
// Start:
//
// cargo run -- --port 3000
//
// Then in another shell:
//
// curl -i http://127.0.0.1:3000/
// curl -i http://127.0.0.1:3000/health/local
// curl -i http://127.0.0.1:3000/health/gemstone

use gemstone_rs::{web, Config};
use std::env;
use std::error::Error;
use std::io::{Error as IoError, ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    let options = Options::parse(env::args().skip(1))?;
    if options.routes {
        print_routes(&options);
        return Ok(());
    }

    let listener = TcpListener::bind(format!("{}:{}", options.host, options.port))?;
    println!(
        "gemstone-rs HTTP example running at http://{}/",
        listener.local_addr()?
    );
    println!("GET /");
    println!("GET /health/local");
    println!("GET /health/gemstone");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                if let Err(err) = handle_connection(&mut stream) {
                    eprintln!("request failed: {err}");
                }
            }
            Err(err) => eprintln!("connection failed: {err}"),
        }
        if options.once {
            break;
        }
    }
    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
struct Options {
    host: String,
    port: u16,
    once: bool,
    routes: bool,
}

impl Options {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, Box<dyn Error>> {
        let mut host = "127.0.0.1".to_string();
        let mut port = 3000;
        let mut once = false;
        let mut routes = false;
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--host" => {
                    host = args
                        .next()
                        .ok_or_else(|| invalid_input("missing value after --host"))?;
                }
                "--port" => {
                    let raw = args
                        .next()
                        .ok_or_else(|| invalid_input("missing value after --port"))?;
                    port = raw.parse()?;
                }
                "--once" => once = true,
                "--routes" => routes = true,
                "-h" | "--help" => {
                    print_usage();
                    std::process::exit(0);
                }
                other => return Err(invalid_input(format!("unknown option: {other}")).into()),
            }
        }

        Ok(Self {
            host,
            port,
            once,
            routes,
        })
    }
}

fn invalid_input(message: impl Into<String>) -> IoError {
    IoError::new(ErrorKind::InvalidInput, message.into())
}

fn print_usage() {
    println!("usage: cargo run -- [--host <host>] [--port <port>] [--once] [--routes]");
}

fn print_routes(options: &Options) {
    println!("gemstone-rs HTTP service example");
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

fn handle_connection(stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut buffer = [0; 4096];
    let read = stream.read(&mut buffer)?;
    if read == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buffer[..read]);
    let Some((method, path)) = parse_request_line(&request) else {
        return write_response(stream, web::bad_request_response());
    };
    if method != "GET" {
        return write_response(stream, web::method_not_allowed_response());
    }

    let response = match path {
        "/" => web::index_response("gemstone-rs HTTP service example"),
        "/health/local" => web::local_health_response(),
        "/health/gemstone" => web::gemstone_health_response_once(Config::from_env()?),
        _ => web::not_found_response(),
    };
    write_response(stream, response)
}

fn parse_request_line(request: &str) -> Option<(&str, &str)> {
    let line = request.lines().next()?;
    let mut parts = line.split_whitespace();
    let method = parts.next()?;
    let path = parts.next()?;
    Some((method, path))
}

fn write_response(stream: &mut TcpStream, response: web::JsonResponse) -> Result<(), Box<dyn Error>> {
    let reason = match response.status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        _ => "OK",
    };
    let response = format!(
        "HTTP/1.1 {} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        response.status,
        response.body.len(),
        response.body
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}
