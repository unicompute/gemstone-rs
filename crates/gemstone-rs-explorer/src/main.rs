use gemstone_rs::{
    browser::{Browser, ALL_PROTOCOLS},
    codegen::{self, DEFAULT_CONFIG_PATH},
    BridgeKeyType, Config, Oop, Session, Value,
};
use std::env;
use std::error::Error as StdError;
use std::fmt;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
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
            let dictionaries = Browser::new(session).dictionaries()?;
            Ok(format!(
                r#"{{"success":true,"dictionaries":{}}}"#,
                json_string_array(dictionaries)
            ))
        }),
        "/api/browse/classes" => {
            let Some(dictionary) = route.non_empty_query("dictionary") else {
                return Response::json(400, r#"{"error":"missing dictionary"}"#.to_string());
            };
            live_json(|session| {
                let classes = Browser::new(session).classes(&dictionary)?;
                Ok(format!(
                    r#"{{"success":true,"dictionary":"{}","classes":{}}}"#,
                    escape_json(&dictionary),
                    json_string_array(classes)
                ))
            })
        }
        "/api/browse/protocols" | "/api/browse/categories" => {
            let Some(class_name) = route.non_empty_query("class") else {
                return Response::json(400, r#"{"error":"missing class"}"#.to_string());
            };
            let dictionary = route.query("dictionary").unwrap_or_default();
            let meta = bool_query(route.query("meta").as_deref());
            live_json(|session| {
                let mut protocols = vec![ALL_PROTOCOLS.to_string()];
                protocols.extend(Browser::new(session).protocols(
                    &class_name,
                    meta,
                    &dictionary,
                )?);
                Ok(format!(
                    r#"{{"success":true,"class":"{}","dictionary":"{}","meta":{},"protocols":{}}}"#,
                    escape_json(&class_name),
                    escape_json(&dictionary),
                    meta,
                    json_string_array(protocols)
                ))
            })
        }
        "/api/browse/methods" => {
            let Some(class_name) = route.non_empty_query("class") else {
                return Response::json(400, r#"{"error":"missing class"}"#.to_string());
            };
            let dictionary = route.query("dictionary").unwrap_or_default();
            let protocol = route
                .non_empty_query("protocol")
                .unwrap_or_else(|| ALL_PROTOCOLS.to_string());
            let meta = bool_query(route.query("meta").as_deref());
            live_json(|session| {
                let methods =
                    Browser::new(session).methods(&class_name, &protocol, meta, &dictionary)?;
                Ok(format!(
                    r#"{{"success":true,"class":"{}","dictionary":"{}","meta":{},"protocol":"{}","methods":{}}}"#,
                    escape_json(&class_name),
                    escape_json(&dictionary),
                    meta,
                    escape_json(&protocol),
                    json_string_array(methods)
                ))
            })
        }
        "/api/browse/source" => {
            let Some(class_name) = route.non_empty_query("class") else {
                return Response::json(400, r#"{"error":"missing class"}"#.to_string());
            };
            let dictionary = route.query("dictionary").unwrap_or_default();
            let selector = route.query("selector").unwrap_or_default();
            let meta = bool_query(route.query("meta").as_deref());
            live_json(|session| {
                let source =
                    Browser::new(session).source(&class_name, &selector, meta, &dictionary)?;
                Ok(format!(
                    r#"{{"success":true,"class":"{}","dictionary":"{}","meta":{},"selector":"{}","source":"{}"}}"#,
                    escape_json(&class_name),
                    escape_json(&dictionary),
                    meta,
                    escape_json(&selector),
                    escape_json(&source)
                ))
            })
        }
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
        "/api/codegen/sample" => Response::json(
            200,
            format!(
                r#"{{"success":true,"config":"{}"}}"#,
                escape_json(codegen::sample_config())
            ),
        ),
        "/api/codegen/discover-mapping" => codegen_discover_mapping_response(&route),
        "/api/codegen/preview" => codegen_preview_response(&route),
        "/api/codegen/diff" => codegen_diff_response(&route),
        "/api/codegen/check" => codegen_check_response(&route),
        "/api/codegen/generate" => {
            if config.read_only {
                return Response::json(
                    403,
                    r#"{"error":"codegen generate disabled; restart with --allow-write"}"#
                        .to_string(),
                );
            }
            codegen_generate_response(&route)
        }
        "/api/bridge/root" => live_json(|session| {
            let (name, oop, identity_id) = {
                let root = session.bridge_root()?;
                (root.name().to_string(), root.oop(), root.identity_id())
            };
            Ok(format!(
                r#"{{"success":true,"name":"{}","oop":{},"identityId":{},"identityMapSize":{}}}"#,
                escape_json(&name),
                oop.raw(),
                identity_id,
                session.identity_map_len()
            ))
        }),
        "/api/bridge/get" => {
            let Some(key) = route.non_empty_query("key") else {
                return Response::json(400, r#"{"error":"missing key"}"#.to_string());
            };
            let key_type = bridge_key_type_query(route.query("key_type").as_deref());
            live_json(|session| {
                let mut root = session.bridge_root()?;
                let oop = root.get_oop_with_key_type(&key, key_type)?;
                drop(root);
                inspect_oop(session, oop)
            })
        }
        "/api/bridge/mapping-config" => {
            let mapped = route
                .non_empty_query("mapped")
                .unwrap_or_else(|| "BookingDraft".to_string());
            Response::json(
                200,
                format!(
                    r#"{{"success":true,"config":"{}"}}"#,
                    escape_json(&sample_mapping_config(&mapped))
                ),
            )
        }
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

fn bool_query(value: Option<&str>) -> bool {
    matches!(
        value.map(str::trim).map(str::to_ascii_lowercase).as_deref(),
        Some("1" | "true" | "yes" | "on")
    )
}

fn codegen_config_path(route: &Route) -> PathBuf {
    route
        .non_empty_query("config")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_PATH))
}

fn codegen_preview_response(route: &Route) -> Response {
    let path = codegen_config_path(route);
    match codegen::Config::from_file(&path).map(|config| codegen::generate(&config)) {
        Ok(generated) => Response::json(
            200,
            format!(
                r#"{{"success":true,"config":"{}","output":"{}","source":"{}"}}"#,
                escape_json(&path.display().to_string()),
                escape_json(&generated.output.display().to_string()),
                escape_json(&generated.source)
            ),
        ),
        Err(err) => codegen_error_response(err),
    }
}

fn codegen_check_response(route: &Route) -> Response {
    let path = codegen_config_path(route);
    match codegen::Config::from_file(&path).and_then(|config| codegen::check(&config)) {
        Ok(report) => Response::json(
            200,
            format!(
                r#"{{"success":true,"config":"{}","output":"{}","exists":{},"upToDate":{}}}"#,
                escape_json(&path.display().to_string()),
                escape_json(&report.output.display().to_string()),
                report.exists,
                report.up_to_date
            ),
        ),
        Err(err) => codegen_error_response(err),
    }
}

fn codegen_diff_response(route: &Route) -> Response {
    let path = codegen_config_path(route);
    match codegen::Config::from_file(&path).and_then(|config| codegen::diff(&config)) {
        Ok(report) => Response::json(
            200,
            format!(
                r#"{{"success":true,"config":"{}","output":"{}","exists":{},"upToDate":{},"diff":"{}"}}"#,
                escape_json(&path.display().to_string()),
                escape_json(&report.output.display().to_string()),
                report.exists,
                report.up_to_date,
                escape_json(&report.diff)
            ),
        ),
        Err(err) => codegen_error_response(err),
    }
}

fn codegen_generate_response(route: &Route) -> Response {
    let path = codegen_config_path(route);
    match codegen::Config::from_file(&path).and_then(|config| codegen::generate_to_file(&config)) {
        Ok(generated) => Response::json(
            200,
            format!(
                r#"{{"success":true,"config":"{}","output":"{}","bytes":{}}}"#,
                escape_json(&path.display().to_string()),
                escape_json(&generated.output.display().to_string()),
                generated.source.len()
            ),
        ),
        Err(err) => codegen_error_response(err),
    }
}

fn codegen_discover_mapping_response(route: &Route) -> Response {
    let path = codegen_config_path(route);
    let mapped_name = route
        .non_empty_query("mapped")
        .unwrap_or_else(|| "BookingDraft".to_string());
    let class_name = route
        .non_empty_query("class")
        .unwrap_or_else(|| "Object".to_string());
    let Ok(class_ref) = codegen::ClassRef::parse(&class_name) else {
        return Response::json(400, r#"{"error":"invalid class"}"#.to_string());
    };
    live_json(|session| {
        let config = codegen::discover_mapping(session, path.clone(), &mapped_name, &class_ref)
            .map_err(|err| gemstone_rs::Error::Mapping {
                field: "codegen discover-mapping".to_string(),
                expected: "GemStone class field metadata",
                actual: err.to_string(),
            })?;
        Ok(format!(
            r#"{{"success":true,"class":"{}","mapped":"{}","config":"{}"}}"#,
            escape_json(&class_ref.display_name()),
            escape_json(&mapped_name),
            escape_json(&codegen::config_source(&config))
        ))
    })
}

fn bridge_key_type_query(value: Option<&str>) -> BridgeKeyType {
    match value.map(str::trim).map(str::to_ascii_lowercase).as_deref() {
        Some("symbol") => BridgeKeyType::Symbol,
        _ => BridgeKeyType::String,
    }
}

fn sample_mapping_config(mapped: &str) -> String {
    format!(
        "mapped = {mapped} | doc=Typed payload stored under GemStoneRsBridgeRoot.\nfield = {mapped}.name | type=String | key=name | key_type=String\nfield = {mapped}.amount | type=SmallInt | key=amount | key_type=String\nfield = {mapped}.tags | type=Vec<String> | key=tags | key_type=String\n"
    )
}

fn codegen_error_response(err: codegen::Error) -> Response {
    Response::json(
        500,
        format!(
            r#"{{"success":false,"error":"{}"}}"#,
            escape_json(&err.to_string())
        ),
    )
}

fn json_string_array(values: impl IntoIterator<Item = String>) -> String {
    let mut result = String::from("[");
    for (index, value) in values.into_iter().enumerate() {
        if index > 0 {
            result.push(',');
        }
        result.push('"');
        result.push_str(&escape_json(&value));
        result.push('"');
    }
    result.push(']');
    result
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

    fn non_empty_query(&self, name: &str) -> Option<String> {
        self.query(name).filter(|value| !value.trim().is_empty())
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
<li><a href="/api/browse/classes?dictionary=UserGlobals">/api/browse/classes?dictionary=UserGlobals</a></li>
<li><a href="/api/browse/protocols?class=Object">/api/browse/protocols?class=Object</a></li>
<li><a href="/api/browse/methods?class=Object&amp;protocol=--%20all%20--">/api/browse/methods?class=Object&amp;protocol=-- all --</a></li>
<li><a href="/api/browse/source?class=Object">/api/browse/source?class=Object</a></li>
<li><a href="/api/codegen/sample">/api/codegen/sample</a></li>
<li><a href="/api/codegen/discover-mapping?mapped=BookingDraft&amp;class=Object">/api/codegen/discover-mapping?mapped=BookingDraft&amp;class=Object</a></li>
<li><a href="/api/codegen/preview?config=examples/codegen/gemstone-rs.codegen">/api/codegen/preview?config=examples/codegen/gemstone-rs.codegen</a></li>
<li><a href="/api/codegen/diff?config=examples/codegen/gemstone-rs.codegen">/api/codegen/diff?config=examples/codegen/gemstone-rs.codegen</a></li>
<li><a href="/api/codegen/check?config=examples/codegen/gemstone-rs.codegen">/api/codegen/check?config=examples/codegen/gemstone-rs.codegen</a></li>
<li><a href="/api/bridge/root">/api/bridge/root</a></li>
<li><a href="/api/bridge/get?key=BookingDraft">/api/bridge/get?key=BookingDraft</a></li>
<li><a href="/api/bridge/mapping-config?mapped=BookingDraft">/api/bridge/mapping-config?mapped=BookingDraft</a></li>
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

    #[test]
    fn browse_classes_requires_dictionary() {
        let response = handle_request(
            "GET /api/browse/classes HTTP/1.1",
            &ExplorerConfig::default(),
        );
        assert_eq!(response.status, 400);
        assert!(response.body.contains("missing dictionary"));
    }

    #[test]
    fn browse_methods_requires_class_without_live_gemstone() {
        let response = handle_request(
            "GET /api/browse/methods?dictionary=UserGlobals HTTP/1.1",
            &ExplorerConfig::default(),
        );
        assert_eq!(response.status, 400);
        assert!(response.body.contains("missing class"));
    }

    #[test]
    fn bool_query_accepts_common_true_values() {
        assert!(bool_query(Some("1")));
        assert!(bool_query(Some("true")));
        assert!(bool_query(Some("YES")));
        assert!(!bool_query(Some("0")));
        assert!(!bool_query(None));
    }

    #[test]
    fn json_string_array_escapes_values() {
        let json = json_string_array(vec![
            "plain".to_string(),
            "quote\"newline\n".to_string(),
            "slash\\".to_string(),
        ]);
        assert_eq!(json, r#"["plain","quote\"newline\n","slash\\"]"#);
    }

    #[test]
    fn codegen_sample_endpoint_returns_config_text() {
        let response = handle_request(
            "GET /api/codegen/sample HTTP/1.1",
            &ExplorerConfig::default(),
        );
        assert_eq!(response.status, 200);
        assert!(response.body.contains("gemstone-rs codegen config"));
    }

    #[test]
    fn codegen_generate_is_disabled_by_default() {
        let response = handle_request(
            "GET /api/codegen/generate HTTP/1.1",
            &ExplorerConfig::default(),
        );
        assert_eq!(response.status, 403);
        assert!(response.body.contains("allow-write"));
    }

    #[test]
    fn codegen_diff_requires_config_file() {
        let response = handle_request(
            "GET /api/codegen/diff?config=missing.codegen HTTP/1.1",
            &ExplorerConfig::default(),
        );
        assert_eq!(response.status, 500);
        assert!(response.body.contains(r#""success":false"#));
    }
}
