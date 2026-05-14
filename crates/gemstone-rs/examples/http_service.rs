// Requires a live GemStone/S stone for GET /health/gemstone.
//
// This example intentionally uses only the Rust standard library. It proves
// the service shape without adding Axum, Actix, Tokio, or serde dependencies to
// the workspace.
//
// Try the route map without starting a server:
//
// cargo run -p gemstone-rs --example http_service -- --routes
//
// Start the local service:
//
// cargo run -p gemstone-rs --example http_service -- --port 3000
//
// Then in another shell:
//
// curl -i http://127.0.0.1:3000/
// curl -i http://127.0.0.1:3000/health/local
// curl -i http://127.0.0.1:3000/health/gemstone

use gemstone_rs::{Config, Session, Value};
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
    println!(
        "usage: cargo run -p gemstone-rs --example http_service -- [--host <host>] [--port <port>] [--once] [--routes]"
    );
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
        "  cargo run -p gemstone-rs --example http_service -- --host {} --port {}",
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
        return write_json(stream, 400, r#"{"error":"bad request"}"#);
    };
    if method != "GET" {
        return write_json(stream, 405, r#"{"error":"method not allowed"}"#);
    }

    match path {
        "/" => write_json(
            stream,
            200,
            r#"{"name":"gemstone-rs HTTP service example","endpoints":{"local":"/health/local","gemstone":"/health/gemstone"}}"#,
        ),
        "/health/local" => write_json(stream, 200, r#"{"ok":true}"#),
        "/health/gemstone" => {
            let (status, body) = gemstone_health_response();
            write_json(stream, status, &body)
        }
        _ => write_json(stream, 404, r#"{"error":"not found"}"#),
    }
}

fn parse_request_line(request: &str) -> Option<(&str, &str)> {
    let line = request.lines().next()?;
    let mut parts = line.split_whitespace();
    let method = parts.next()?;
    let path = parts.next()?;
    Some((method, path))
}

fn gemstone_health_response() -> (u16, String) {
    match gemstone_health_value() {
        Ok(value) => (200, format!(r#"{{"result":{value}}}"#)),
        Err(err) => (
            500,
            format!(r#"{{"error":"{}"}}"#, json_escape(&err.to_string())),
        ),
    }
}

fn gemstone_health_value() -> Result<i64, Box<dyn Error>> {
    let mut session = Session::login(Config::from_env()?)?;
    let value = session.eval("3 + 4")?;
    let Value::SmallInt(value) = value else {
        return Err(invalid_input("GemStone health check returned a non-SmallInt value").into());
    };
    session.logout()?;
    Ok(value)
}

fn write_json(stream: &mut TcpStream, status: u16, body: &str) -> Result<(), Box<dyn Error>> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        _ => "OK",
    };
    let response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes())?;
    stream.flush()?;
    Ok(())
}

fn json_escape(value: &str) -> String {
    let mut output = String::new();
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            other => output.push(other),
        }
    }
    output
}
