use gemstone_rs::{Config, Oop, Session, Value};
use std::env;
use std::error::Error as StdError;
use std::fmt;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::process::ExitCode;

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(1)
        }
    }
}

fn run(args: Vec<String>) -> Result<(), ExplorerError> {
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "-h" | "--help"))
    {
        println!("{}", usage());
        return Ok(());
    }

    let config = ExplorerConfig::parse(&args)?;
    let listener = TcpListener::bind(config.addr())?;
    eprintln!(
        "gemstone-rs explorer listening at http://{}/",
        config.addr()
    );
    eprintln!(
        "read_only={}, allow_eval={}",
        config.read_only, config.allow_eval
    );

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(err) = handle_stream(stream, &config) {
                    eprintln!("request error: {err}");
                }
            }
            Err(err) => eprintln!("accept error: {err}"),
        }
    }
    Ok(())
}

fn handle_stream(mut stream: TcpStream, config: &ExplorerConfig) -> Result<(), ExplorerError> {
    let mut buffer = [0_u8; 8192];
    let read = stream.read(&mut buffer)?;
    let request = String::from_utf8_lossy(&buffer[..read]);
    let Some(request_line) = request.lines().next() else {
        return Ok(());
    };

    let response = handle_request(request_line, config);
    stream.write_all(response.to_http().as_bytes())?;
    Ok(())
}

fn handle_request(request_line: &str, config: &ExplorerConfig) -> Response {
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Response::json(400, r#"{"error":"bad request"}"#.to_string());
    }
    if parts[0] != "GET" {
        return Response::json(405, r#"{"error":"method not allowed"}"#.to_string());
    }

    let route = Route::parse(parts[1]);
    match route.path.as_str() {
        "/" => Response::html(200, landing_html(config)),
        "/health" => Response::json(200, r#"{"status":"ok"}"#.to_string()),
        "/api/config" => Response::json(200, config_json(config)),
        "/api/status" => live_json(|session| {
            let needs_commit = session.needs_commit()?;
            let in_transaction = session.in_transaction()?;
            Ok(format!(
                r#"{{"connected":true,"sessionId":{},"needsCommit":{},"inTransaction":{}}}"#,
                session.session_id(),
                needs_commit,
                in_transaction
            ))
        }),
        "/api/browse/dictionaries" => live_json(|session| {
            let source =
                "System myUserProfile symbolList dictionaries collect: [:dict | dict name]";
            let value = session.eval(source)?;
            value_json(session, value)
        }),
        "/api/inspect" => {
            let Some(raw) = route.query("oop").and_then(|value| parse_oop(&value).ok()) else {
                return Response::json(400, r#"{"error":"missing or invalid oop"}"#.to_string());
            };
            live_json(|session| inspect_oop(session, raw))
        }
        "/api/eval" => {
            if !config.allow_eval {
                return Response::json(
                    403,
                    r#"{"error":"eval disabled; restart with --allow-eval"}"#.to_string(),
                );
            }
            let Some(source) = route.query("source") else {
                return Response::json(400, r#"{"error":"missing source"}"#.to_string());
            };
            live_json(|session| {
                let value = session.eval(&source)?;
                value_json(session, value)
            })
        }
        "/api/codegen/check" => Response::json(
            501,
            r#"{"status":"planned","message":"codegen check is not wired yet"}"#.to_string(),
        ),
        "/api/codegen/generate" => Response::json(
            501,
            r#"{"status":"planned","message":"codegen generate is not wired yet"}"#.to_string(),
        ),
        _ => Response::json(404, r#"{"error":"not found"}"#.to_string()),
    }
}

fn live_json(body: impl FnOnce(&mut Session) -> gemstone_rs::Result<String>) -> Response {
    match Session::login(match Config::from_env() {
        Ok(config) => config,
        Err(err) => {
            return Response::json(
                503,
                format!(
                    r#"{{"connected":false,"error":"{}"}}"#,
                    escape_json(&err.to_string())
                ),
            )
        }
    }) {
        Ok(mut session) => match body(&mut session) {
            Ok(body) => Response::json(200, body),
            Err(err) => Response::json(
                500,
                format!(
                    r#"{{"connected":true,"error":"{}"}}"#,
                    escape_json(&err.to_string())
                ),
            ),
        },
        Err(err) => Response::json(
            503,
            format!(
                r#"{{"connected":false,"error":"{}"}}"#,
                escape_json(&err.to_string())
            ),
        ),
    }
}

fn inspect_oop(session: &mut Session, oop: Oop) -> gemstone_rs::Result<String> {
    let class = session.fetch_class(oop)?;
    let printed = session.perform_oop(oop, "printString", &[])?;
    let print_string = session.fetch_string(printed)?;
    Ok(format!(
        r#"{{"oop":{},"classOop":{},"printString":"{}"}}"#,
        oop.raw(),
        class.raw(),
        escape_json(&print_string)
    ))
}

fn value_json(session: &mut Session, value: Value) -> gemstone_rs::Result<String> {
    Ok(match value {
        Value::Nil => r#"{"type":"nil","value":null}"#.to_string(),
        Value::Bool(value) => format!(r#"{{"type":"bool","value":{value}}}"#),
        Value::SmallInt(value) => format!(r#"{{"type":"smallInt","value":{value}}}"#),
        Value::Char(value) => format!(
            r#"{{"type":"char","value":"{}"}}"#,
            escape_json(&value.to_string())
        ),
        Value::String(value) => format!(r#"{{"type":"string","value":"{}"}}"#, escape_json(&value)),
        Value::Oop(oop) => {
            let printed = session.perform_oop(oop, "printString", &[])?;
            let print_string = session.fetch_string(printed)?;
            format!(
                r#"{{"type":"oop","oop":{},"printString":"{}"}}"#,
                oop.raw(),
                escape_json(&print_string)
            )
        }
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ExplorerConfig {
    host: IpAddr,
    port: u16,
    read_only: bool,
    allow_eval: bool,
}

impl ExplorerConfig {
    fn parse(args: &[String]) -> Result<Self, ExplorerError> {
        let mut config = Self::default();
        let mut index = 0;
        while index < args.len() {
            match args[index].as_str() {
                "--host" => {
                    index += 1;
                    let value = args
                        .get(index)
                        .ok_or_else(|| ExplorerError::usage("missing value for --host"))?;
                    config.host = value
                        .parse()
                        .map_err(|_| ExplorerError::usage(format!("invalid host: {value}")))?;
                }
                "--port" => {
                    index += 1;
                    let value = args
                        .get(index)
                        .ok_or_else(|| ExplorerError::usage("missing value for --port"))?;
                    config.port = value
                        .parse()
                        .map_err(|_| ExplorerError::usage(format!("invalid port: {value}")))?;
                }
                "--allow-eval" => config.allow_eval = true,
                "--allow-write" => config.read_only = false,
                other => return Err(ExplorerError::usage(format!("unknown option: {other}"))),
            }
            index += 1;
        }
        if !config.host.is_loopback() {
            return Err(ExplorerError::usage(
                "gemstone-rs-explorer only binds to loopback addresses",
            ));
        }
        Ok(config)
    }

    fn addr(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.port)
    }
}

impl Default for ExplorerConfig {
    fn default() -> Self {
        Self {
            host: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: 8787,
            read_only: true,
            allow_eval: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Route {
    path: String,
    query: Vec<(String, String)>,
}

impl Route {
    fn parse(target: &str) -> Self {
        let (path, query) = target.split_once('?').unwrap_or((target, ""));
        let query = query
            .split('&')
            .filter(|part| !part.is_empty())
            .map(|part| {
                let (key, value) = part.split_once('=').unwrap_or((part, ""));
                (percent_decode(key), percent_decode(value))
            })
            .collect();
        Self {
            path: path.to_string(),
            query,
        }
    }

    fn query(&self, name: &str) -> Option<String> {
        self.query
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.clone())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Response {
    status: u16,
    content_type: &'static str,
    body: String,
}

impl Response {
    fn json(status: u16, body: String) -> Self {
        Self {
            status,
            content_type: "application/json; charset=utf-8",
            body,
        }
    }

    fn html(status: u16, body: String) -> Self {
        Self {
            status,
            content_type: "text/html; charset=utf-8",
            body,
        }
    }

    fn to_http(&self) -> String {
        format!(
            "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            self.status,
            reason_phrase(self.status),
            self.content_type,
            self.body.len(),
            self.body
        )
    }
}

fn landing_html(config: &ExplorerConfig) -> String {
    format!(
        r#"<!doctype html>
<html>
<head><meta charset="utf-8"><title>gemstone-rs Explorer</title></head>
<body>
<h1>gemstone-rs Explorer</h1>
<p>Local-only GemStone/S explorer.</p>
<ul>
<li><a href="/api/config">/api/config</a></li>
<li><a href="/api/status">/api/status</a></li>
<li><a href="/api/browse/dictionaries">/api/browse/dictionaries</a></li>
<li><a href="/api/inspect?oop=20">/api/inspect?oop=20</a></li>
</ul>
<p>read_only={} allow_eval={}</p>
</body>
</html>"#,
        config.read_only, config.allow_eval
    )
}

fn config_json(config: &ExplorerConfig) -> String {
    format!(
        r#"{{"host":"{}","port":{},"readOnly":{},"allowEval":{},"loopbackOnly":true}}"#,
        config.host, config.port, config.read_only, config.allow_eval
    )
}

fn parse_oop(value: &str) -> Result<Oop, ExplorerError> {
    let raw = if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        u64::from_str_radix(hex, 16)
    } else {
        value.parse()
    }
    .map_err(|_| ExplorerError::usage(format!("invalid OOP: {value}")))?;
    Ok(Oop(raw))
}

fn percent_decode(value: &str) -> String {
    let mut result = String::new();
    let mut chars = value.as_bytes().iter().copied();
    while let Some(byte) = chars.next() {
        match byte {
            b'+' => result.push(' '),
            b'%' => {
                let high = chars.next();
                let low = chars.next();
                if let (Some(high), Some(low)) = (high, low) {
                    if let Ok(decoded) =
                        u8::from_str_radix(&String::from_utf8_lossy(&[high, low]), 16)
                    {
                        result.push(decoded as char);
                    }
                }
            }
            _ => result.push(byte as char),
        }
    }
    result
}

fn escape_json(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            ch if ch.is_control() => result.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => result.push(ch),
        }
    }
    result
}

fn reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        503 => "Service Unavailable",
        _ => "OK",
    }
}

fn usage() -> &'static str {
    "usage: gemstone-rs-explorer [--host 127.0.0.1] [--port 8787] [--allow-eval] [--allow-write]"
}

#[derive(Debug)]
enum ExplorerError {
    Usage(String),
    Io(std::io::Error),
}

impl ExplorerError {
    fn usage(message: impl Into<String>) -> Self {
        Self::Usage(message.into())
    }
}

impl fmt::Display for ExplorerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => write!(f, "{message}"),
            Self::Io(err) => write!(f, "{err}"),
        }
    }
}

impl StdError for ExplorerError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Usage(_) => None,
            Self::Io(err) => Some(err),
        }
    }
}

impl From<std::io::Error> for ExplorerError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn config_defaults_to_loopback_read_only() {
        let config = ExplorerConfig::parse(&[]).unwrap();
        assert_eq!(config.addr(), SocketAddr::from(([127, 0, 0, 1], 8787)));
        assert!(config.read_only);
        assert!(!config.allow_eval);
    }

    #[test]
    fn config_rejects_non_loopback_host() {
        let err = ExplorerConfig::parse(&args(&["--host", "0.0.0.0"])).unwrap_err();
        assert!(err.to_string().contains("loopback"));
    }

    #[test]
    fn config_accepts_eval_flag_and_port() {
        let config = ExplorerConfig::parse(&args(&["--port", "9000", "--allow-eval"])).unwrap();
        assert_eq!(config.port, 9000);
        assert!(config.allow_eval);
    }

    #[test]
    fn parses_routes_and_decodes_query() {
        let route = Route::parse("/api/eval?source=3%20%2B%204");
        assert_eq!(route.path, "/api/eval");
        assert_eq!(route.query("source").as_deref(), Some("3 + 4"));
    }

    #[test]
    fn eval_endpoint_is_disabled_by_default() {
        let response = handle_request(
            "GET /api/eval?source=3%20%2B%204 HTTP/1.1",
            &ExplorerConfig::default(),
        );
        assert_eq!(response.status, 403);
    }

    #[test]
    fn config_endpoint_reports_safe_defaults() {
        let response = handle_request("GET /api/config HTTP/1.1", &ExplorerConfig::default());
        assert_eq!(response.status, 200);
        assert!(response.body.contains(r#""loopbackOnly":true"#));
        assert!(response.body.contains(r#""readOnly":true"#));
    }
}
