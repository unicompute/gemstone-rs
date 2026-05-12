use gemstone_rs::{
    browser::{Browser, ALL_PROTOCOLS},
    codegen::{self, DEFAULT_CONFIG_PATH},
    gci_library_path, BridgeKeySummary, BridgeKeyType, BridgeValue, Config, Oop, Session, Value,
    DEFAULT_BRIDGE_ROOT,
};
use std::env;
use std::error::Error as StdError;
use std::fmt;
use std::fs;
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
    let Some(request) = read_http_request(&mut stream)? else {
        return Ok(());
    };
    let response = handle_http_request(&request, config);
    stream.write_all(response.to_http().as_bytes())?;
    Ok(())
}

fn read_http_request(stream: &mut TcpStream) -> Result<Option<HttpRequest>, ExplorerError> {
    let mut buffer = [0_u8; 8192];
    let mut bytes = Vec::new();
    let body_start = loop {
        let read = stream.read(&mut buffer)?;
        if read == 0 {
            return Ok(None);
        }
        bytes.extend_from_slice(&buffer[..read]);
        if let Some(body_start) = header_body_offset(&bytes) {
            break body_start;
        }
        if bytes.len() > 64 * 1024 {
            return Err(ExplorerError::usage("request headers too large"));
        }
    };

    let headers = String::from_utf8_lossy(&bytes[..body_start]).to_string();
    let content_length = content_length(&headers);
    while bytes.len() < body_start + content_length {
        let read = stream.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&buffer[..read]);
        if bytes.len() > body_start + 1024 * 1024 {
            return Err(ExplorerError::usage("request body too large"));
        }
    }
    if bytes.len() < body_start + content_length {
        return Err(ExplorerError::usage("incomplete request body"));
    }

    let Some(request_line) = headers.lines().next() else {
        return Ok(None);
    };
    let body = String::from_utf8_lossy(&bytes[body_start..body_start + content_length]).to_string();
    Ok(HttpRequest::from_request_line(request_line, body))
}

fn header_body_offset(bytes: &[u8]) -> Option<usize> {
    bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|index| index + 4)
        .or_else(|| {
            bytes
                .windows(2)
                .position(|window| window == b"\n\n")
                .map(|index| index + 2)
        })
}

fn content_length(headers: &str) -> usize {
    headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse().ok())
                .flatten()
        })
        .unwrap_or(0)
}

#[cfg(test)]
fn handle_request(request_line: &str, config: &ExplorerConfig) -> Response {
    let Some(request) = HttpRequest::from_request_line(request_line, String::new()) else {
        return Response::json(400, r#"{"error":"bad request"}"#.to_string());
    };
    handle_http_request(&request, config)
}

fn handle_http_request(request: &HttpRequest, config: &ExplorerConfig) -> Response {
    let route = Route::parse(&request.target);
    if request.method != "GET"
        && !(request.method == "POST" && route.path == "/api/codegen/config/save")
    {
        return Response::json(405, r#"{"error":"method not allowed"}"#.to_string());
    }

    match route.path.as_str() {
        "/" => Response::html(200, landing_html(config)),
        "/health" => Response::json(200, r#"{"status":"ok"}"#.to_string()),
        "/api/config" => Response::json(200, config_json(config)),
        "/api/doctor" => doctor_response(bool_query(route.query("live").as_deref())),
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
        "/api/codegen/config" => codegen_config_response(&route),
        "/api/codegen/config/save" => {
            if config.read_only {
                return Response::json(
                    403,
                    r#"{"error":"codegen config save disabled; restart with --allow-write"}"#
                        .to_string(),
                );
            }
            codegen_config_save_response(&route, &request.body)
        }
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
            let root_name = route
                .non_empty_query("root")
                .unwrap_or_else(|| DEFAULT_BRIDGE_ROOT.to_string());
            let (name, oop, identity_id) = {
                let root = session.bridge_root_named(root_name)?;
                (root.name().to_string(), root.oop(), root.identity_id())
            };
            Ok(format!(
                r#"{{"success":true,"name":"{}","oop":{},"identityId":{}}}"#,
                escape_json(&name),
                oop.raw(),
                identity_id
            ))
        }),
        "/api/bridge/keys" => live_json(|session| {
            let root_name = route
                .non_empty_query("root")
                .unwrap_or_else(|| DEFAULT_BRIDGE_ROOT.to_string());
            let (name, keys) = {
                let mut root = session.bridge_root_named(root_name)?;
                (root.name().to_string(), root.keys()?)
            };
            Ok(format!(
                r#"{{"success":true,"root":"{}","keys":{}}}"#,
                escape_json(&name),
                bridge_keys_json(&keys)
            ))
        }),
        "/api/bridge/get" => {
            let Some(key) = route.non_empty_query("key") else {
                return Response::json(400, r#"{"error":"missing key"}"#.to_string());
            };
            let root_name = route
                .non_empty_query("root")
                .unwrap_or_else(|| DEFAULT_BRIDGE_ROOT.to_string());
            let key_type = bridge_key_type_query(route.query("key_type").as_deref());
            live_json(|session| {
                let mut root = session.bridge_root_named(root_name)?;
                let oop = root.get_oop_with_key_type(&key, key_type)?;
                drop(root);
                inspect_oop(session, oop)
            })
        }
        "/api/bridge/put" => {
            if config.read_only {
                return Response::json(
                    403,
                    r#"{"error":"bridge writes disabled; restart with --allow-write"}"#.to_string(),
                );
            }
            let Some(key) = route.non_empty_query("key") else {
                return Response::json(400, r#"{"error":"missing key"}"#.to_string());
            };
            let Some(raw_value) = route.non_empty_query("value") else {
                return Response::json(400, r#"{"error":"missing value"}"#.to_string());
            };
            let root_name = route
                .non_empty_query("root")
                .unwrap_or_else(|| DEFAULT_BRIDGE_ROOT.to_string());
            let key_type = bridge_key_type_query(route.query("key_type").as_deref());
            let value_type = bridge_value_type_query(route.query("value_type").as_deref());
            let value = match value_type.parse_value(&raw_value) {
                Ok(value) => value,
                Err(message) => return Response::json(400, format!(r#"{{"error":"{message}"}}"#)),
            };
            live_json(|session| {
                let (root_name, oop) = {
                    let mut root = session.bridge_root_named(root_name)?;
                    let oop = root.put_with_key_type(&key, key_type, value)?;
                    root.commit()?;
                    (root.name().to_string(), oop)
                };
                Ok(format!(
                    r#"{{"success":true,"root":"{}","key":"{}","keyType":"{}","valueType":"{}","oop":{}}}"#,
                    escape_json(&root_name),
                    escape_json(&key),
                    key_type.config_name(),
                    value_type.name(),
                    oop.raw()
                ))
            })
        }
        "/api/bridge/remove" => {
            if config.read_only {
                return Response::json(
                    403,
                    r#"{"error":"bridge writes disabled; restart with --allow-write"}"#.to_string(),
                );
            }
            let Some(key) = route.non_empty_query("key") else {
                return Response::json(400, r#"{"error":"missing key"}"#.to_string());
            };
            let root_name = route
                .non_empty_query("root")
                .unwrap_or_else(|| DEFAULT_BRIDGE_ROOT.to_string());
            let key_type = bridge_key_type_query(route.query("key_type").as_deref());
            live_json(|session| {
                let (root_name, oop) = {
                    let mut root = session.bridge_root_named(root_name)?;
                    let oop = root.remove_with_key_type(&key, key_type)?;
                    root.commit()?;
                    (root.name().to_string(), oop)
                };
                Ok(format!(
                    r#"{{"success":true,"root":"{}","key":"{}","keyType":"{}","removedOop":{}}}"#,
                    escape_json(&root_name),
                    escape_json(&key),
                    key_type.config_name(),
                    oop.raw()
                ))
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

fn doctor_response(live: bool) -> Response {
    let mut ok = true;
    let mut gci_json = String::from(r#"{"ok":false,"checked":false}"#);
    let mut live_json = if live {
        String::from(r#"{"ok":false,"checked":true,"error":"config incomplete"}"#)
    } else {
        String::from(r#"{"ok":true,"checked":false}"#)
    };

    let (config_json, config) = match Config::from_env() {
        Ok(config) => {
            let config_json = format!(
                r#"{{"ok":true,"stone":"{}","stoneNrs":"{}","host":"{}","netldi":"{}","username":"{}","gemService":"{}"}}"#,
                escape_json(&config.stone),
                escape_json(&config.stone_nrs()),
                escape_json(&config.host),
                escape_json(&config.netldi),
                escape_json(&config.username),
                escape_json(&config.gem_service)
            );
            (config_json, Some(config))
        }
        Err(err) => {
            ok = false;
            (
                format!(
                    r#"{{"ok":false,"error":"{}"}}"#,
                    escape_json(&err.to_string())
                ),
                None,
            )
        }
    };

    if let Some(config) = config {
        match gci_library_path(&config) {
            Ok(path) => {
                gci_json = format!(
                    r#"{{"ok":true,"checked":true,"path":"{}"}}"#,
                    escape_json(&path.display().to_string())
                );
            }
            Err(err) => {
                ok = false;
                gci_json = format!(
                    r#"{{"ok":false,"checked":true,"error":"{}"}}"#,
                    escape_json(&err.to_string())
                );
            }
        }

        if live {
            match Session::login(config).and_then(|mut session| session.eval("3 + 4")) {
                Ok(Value::SmallInt(7)) => {
                    live_json = r#"{"ok":true,"checked":true,"result":7}"#.to_string();
                }
                Ok(value) => {
                    ok = false;
                    live_json = format!(
                        r#"{{"ok":false,"checked":true,"error":"expected 7","actual":"{}"}}"#,
                        escape_json(&format!("{value:?}"))
                    );
                }
                Err(err) => {
                    ok = false;
                    live_json = format!(
                        r#"{{"ok":false,"checked":true,"error":"{}"}}"#,
                        escape_json(&err.to_string())
                    );
                }
            }
        }
    }

    Response::json(
        if ok { 200 } else { 503 },
        format!(
            r#"{{"success":{},"environment":{},"config":{},"gciLibrary":{},"live":{}}}"#,
            ok,
            environment_json(),
            config_json,
            gci_json,
            live_json
        ),
    )
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

fn environment_json() -> String {
    let entries = [
        ("GS_LIB_PATH", false),
        ("GS_LIB", false),
        ("GEMSTONE", false),
        ("GS_STONE", false),
        ("GS_STONE_NAME", false),
        ("GS_HOST", false),
        ("GS_NETLDI", false),
        ("GS_GEM_SERVICE", false),
        ("GS_USERNAME", false),
        ("GS_PASSWORD", true),
        ("GS_HOST_USERNAME", false),
        ("GS_HOST_PASSWORD", true),
    ];
    let mut output = String::from("{");
    for (index, (name, secret)) in entries.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push('"');
        output.push_str(name);
        output.push_str(r#"":"#);
        match env::var(name) {
            Ok(value) if value.is_empty() => output.push_str(r#"{"status":"empty"}"#),
            Ok(_) if *secret => output.push_str(r#"{"status":"set","masked":true}"#),
            Ok(value) => output.push_str(&format!(
                r#"{{"status":"set","value":"{}"}}"#,
                escape_json(&value)
            )),
            Err(_) => output.push_str(r#"{"status":"unset"}"#),
        }
    }
    output.push('}');
    output
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

fn bridge_keys_json(keys: &[BridgeKeySummary]) -> String {
    let mut result = String::from("[");
    for (index, key) in keys.iter().enumerate() {
        if index > 0 {
            result.push(',');
        }
        result.push_str(&format!(
            r#"{{"oop":{},"classOop":{},"printString":"{}","identityId":{}}}"#,
            key.oop.raw(),
            key.class_oop.raw(),
            escape_json(&key.print_string),
            key.identity_id
        ));
    }
    result.push(']');
    result
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

fn codegen_config_response(route: &Route) -> Response {
    let path = codegen_config_path(route);
    match fs::read_to_string(&path) {
        Ok(source) => Response::json(
            200,
            format!(
                r#"{{"success":true,"config":"{}","source":"{}"}}"#,
                escape_json(&path.display().to_string()),
                escape_json(&source)
            ),
        ),
        Err(err) => Response::json(
            404,
            format!(
                r#"{{"success":false,"config":"{}","error":"{}"}}"#,
                escape_json(&path.display().to_string()),
                escape_json(&err.to_string())
            ),
        ),
    }
}

fn codegen_config_save_response(route: &Route, body: &str) -> Response {
    let path = codegen_config_path(route);
    let Some(source) = route
        .query("source")
        .or_else(|| (!body.is_empty()).then(|| body.to_string()))
    else {
        return Response::json(400, r#"{"error":"missing source"}"#.to_string());
    };
    if let Err(err) = codegen::Config::parse(&source, Some(&path)) {
        return Response::json(
            400,
            format!(
                r#"{{"success":false,"config":"{}","error":"{}"}}"#,
                escape_json(&path.display().to_string()),
                escape_json(&err.to_string())
            ),
        );
    }
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        if let Err(err) = fs::create_dir_all(parent) {
            return Response::json(
                500,
                format!(
                    r#"{{"success":false,"config":"{}","error":"{}"}}"#,
                    escape_json(&path.display().to_string()),
                    escape_json(&err.to_string())
                ),
            );
        }
    }
    match fs::write(&path, &source) {
        Ok(()) => Response::json(
            200,
            format!(
                r#"{{"success":true,"config":"{}","bytes":{},"source":"{}"}}"#,
                escape_json(&path.display().to_string()),
                source.len(),
                escape_json(&source)
            ),
        ),
        Err(err) => Response::json(
            500,
            format!(
                r#"{{"success":false,"config":"{}","error":"{}"}}"#,
                escape_json(&path.display().to_string()),
                escape_json(&err.to_string())
            ),
        ),
    }
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BridgeValueType {
    String,
    SmallInt,
    Bool,
}

impl BridgeValueType {
    fn name(self) -> &'static str {
        match self {
            Self::String => "String",
            Self::SmallInt => "SmallInt",
            Self::Bool => "Bool",
        }
    }

    fn parse_value(self, value: &str) -> Result<BridgeValue, String> {
        match self {
            Self::String => Ok(BridgeValue::from(value.to_string())),
            Self::SmallInt => value
                .parse::<i64>()
                .map(BridgeValue::from)
                .map_err(|_| format!("expected SmallInt value, got {}", escape_json(value))),
            Self::Bool => parse_bridge_bool(value).map(BridgeValue::from),
        }
    }
}

fn bridge_value_type_query(value: Option<&str>) -> BridgeValueType {
    match value.map(str::trim).map(str::to_ascii_lowercase).as_deref() {
        Some("smallint" | "small_int" | "int" | "integer") => BridgeValueType::SmallInt,
        Some("bool" | "boolean") => BridgeValueType::Bool,
        _ => BridgeValueType::String,
    }
}

fn parse_bridge_bool(value: &str) -> Result<bool, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(format!("expected Bool value, got {}", escape_json(value))),
    }
}

fn sample_mapping_config(mapped: &str) -> String {
    codegen::sample_mapping_config(mapped)
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
struct HttpRequest {
    method: String,
    target: String,
    body: String,
}

impl HttpRequest {
    fn from_request_line(request_line: &str, body: String) -> Option<Self> {
        let mut parts = request_line.split_whitespace();
        let method = parts.next()?.to_string();
        let target = parts.next()?.to_string();
        Some(Self {
            method,
            target,
            body,
        })
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
    let write_note = if config.read_only {
        "Writes disabled. Restart with --allow-write for codegen generate or BridgeRoot edits."
    } else {
        "Writes enabled for this loopback-only explorer."
    };
    let eval_note = if config.allow_eval {
        "Eval enabled."
    } else {
        "Eval disabled. Restart with --allow-eval for workspace eval."
    };
    let mut html = String::from(
        r#"<!doctype html>
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>gemstone-rs Explorer</title>
<style>
:root { color-scheme: light dark; font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; }
body { margin: 0; background: #f7f7f8; color: #24292f; }
header { background: #9f1722; color: white; padding: 18px 24px; }
main { display: grid; grid-template-columns: minmax(220px, 320px) 1fr; gap: 16px; padding: 16px; }
section { background: white; border: 1px solid #d0d7de; border-radius: 8px; padding: 14px; }
h1, h2 { margin: 0 0 10px; }
h1 { font-size: 24px; }
h2 { font-size: 16px; }
button, input, select, textarea { font: inherit; }
button { border: 1px solid #9f1722; background: #9f1722; color: white; border-radius: 6px; padding: 7px 10px; cursor: pointer; }
button.secondary { background: white; color: #9f1722; }
input, select, textarea { box-sizing: border-box; width: 100%; border: 1px solid #d0d7de; border-radius: 6px; padding: 7px; margin: 4px 0 8px; }
textarea { min-height: 150px; font-family: ui-monospace, SFMono-Regular, Consolas, "Liberation Mono", monospace; resize: vertical; }
.row { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
.actions { display: flex; flex-wrap: wrap; gap: 8px; margin: 8px 0; }
.status { display: flex; flex-wrap: wrap; gap: 8px; margin-top: 8px; }
.pill { border: 1px solid rgba(255,255,255,.45); border-radius: 999px; padding: 4px 8px; font-size: 12px; }
.list { max-height: 420px; overflow: auto; }
.item { display: block; width: 100%; text-align: left; border-color: #d0d7de; background: white; color: #24292f; margin-bottom: 4px; }
pre { min-height: 220px; overflow: auto; white-space: pre-wrap; background: #0d1117; color: #e6edf3; border-radius: 8px; padding: 12px; }
.panes { display: grid; grid-template-columns: 1fr; gap: 8px; }
.pane-title { color: #57606a; font-size: 12px; margin: 8px 0 4px; }
.detail { min-height: 180px; }
.diff span { display: block; min-height: 1em; }
.diff-add { color: #7ee787; }
.diff-remove { color: #ff7b72; }
.diff-meta { color: #79c0ff; }
.side-by-side { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; margin-top: 8px; }
.side-cell { white-space: pre-wrap; background: #161b22; border-radius: 6px; padding: 6px 8px; min-height: 1em; }
.side-old { color: #ffb3ad; }
.side-new { color: #b7f7c8; }
.side-meta { grid-column: 1 / -1; color: #79c0ff; background: #161b22; border-radius: 6px; padding: 6px 8px; }
@media (max-width: 760px) { main { grid-template-columns: 1fr; } .row { grid-template-columns: 1fr; } }
</style>
</head>
<body>
<header>
<h1>gemstone-rs Explorer</h1>
<div class="status">
"#,
    );
    html.push_str(&format!(
        r#"<span class="pill">read_only={}</span><span class="pill">allow_eval={}</span><span class="pill">{}</span><span class="pill">{}</span>"#,
        config.read_only, config.allow_eval, write_note, eval_note
    ));
    html.push_str(
        r#"
</div>
</header>
<main>
<section>
<h2>Browse</h2>
<div class="actions">
<button onclick="loadDictionaries()">Dictionaries</button>
<button class="secondary" onclick="loadClasses()">Classes</button>
<button class="secondary" onclick="loadProtocols()">Protocols</button>
<button class="secondary" onclick="loadMethods()">Methods</button>
<button class="secondary" onclick="loadSource()">Source</button>
</div>
<label>Dictionary<input id="dictionary" value="UserGlobals"></label>
<label>Class<input id="className" value="Object"></label>
<label>Protocol<input id="protocol" value="-- all --"></label>
<label>Selector<input id="selector" value="printString"></label>
<div id="items" class="list"></div>
</section>
<section>
<h2>BridgeRoot and Codegen</h2>
<div class="actions">
<button onclick="callApi('/api/doctor?live=1')">Doctor Live</button>
<button onclick="callApi('/api/status')">Status</button>
<button onclick="callApi('/api/bridge/root')">BridgeRoot</button>
<button onclick="loadBridgeKeys()">Keys</button>
</div>
<div class="row">
<label>Bridge key<input id="bridgeKey" value="WorkbenchDraft"></label>
<label>Bridge value<input id="bridgeValue" value="hello"></label>
</div>
<div class="row">
<label>Bridge key type<select id="bridgeKeyType"><option>String</option><option>Symbol</option></select></label>
<label>Bridge value type<select id="bridgeValueType"><option>String</option><option>SmallInt</option><option>Bool</option></select></label>
</div>
<div class="actions">
<button class="secondary" onclick="getBridgeValue()">Get</button>
<button class="secondary" onclick="putBridgeValue()">Put Value</button>
<button class="secondary" onclick="removeBridgeValue()">Remove</button>
</div>
<h2>Codegen Workflow</h2>
<label>Config path<input id="codegenConfig" value="examples/codegen/gemstone-rs.codegen"></label>
<label>Config editor<textarea id="configEditor" spellcheck="false">Load a config, sample config, or discovered mapping proposal.</textarea></label>
<div class="row">
<label>Mapped Rust type<input id="mappedName" value="BookingDraft"></label>
<label>GemStone class<input id="mappingClass" value="Object"></label>
</div>
<div class="actions">
<button class="secondary" onclick="loadCodegenConfig()">Load Config</button>
<button class="secondary" onclick="saveCodegenConfig()">Save Config</button>
<button class="secondary" onclick="codegenSample()">Sample Config</button>
<button class="secondary" onclick="discoverMappingConfig()">Discover Mapping</button>
<button class="secondary" onclick="codegenPreview()">Preview</button>
<button class="secondary" onclick="codegenDiff()">Diff</button>
<button class="secondary" onclick="codegenCheck()">Check</button>
<button class="secondary" onclick="codegenGenerate()">Generate</button>
<button class="secondary" onclick="clearSavedFields()">Clear Saved Fields</button>
</div>
<div class="panes">
<div>
<div class="pane-title">Response</div>
<pre id="output">Ready.</pre>
</div>
<div>
<div class="pane-title">Generated Source / Config / Diff</div>
<pre id="detail" class="detail">Run Preview, Diff, Sample Config, or Discover Mapping.</pre>
</div>
</div>
</section>
</main>
<script>
const out = document.getElementById('output');
const detail = document.getElementById('detail');
const items = document.getElementById('items');
function q(id) { return encodeURIComponent(document.getElementById(id).value); }
function bridgeQuery() { return 'key=' + q('bridgeKey') + '&key_type=' + q('bridgeKeyType'); }
function codegenQuery() { return 'config=' + q('codegenConfig'); }
async function callApi(path, options = {}) {
  const method = options.method || 'GET';
  out.textContent = method + ' ' + path + '\n';
  detail.className = 'detail';
  detail.textContent = '';
  const response = await fetch(path, options);
  const text = await response.text();
  try {
    const data = JSON.parse(text);
    out.textContent += JSON.stringify(data, null, 2);
    renderDetail(data);
    return data;
  }
  catch {
    out.textContent += text;
    detail.textContent = text;
    return { raw: text };
  }
}
function renderDetail(data) {
  if (typeof data.diff === 'string') {
    renderDiff(data.diff || 'No generated output changes.');
  } else if (typeof data.source === 'string') {
    detail.textContent = data.source;
  } else if (typeof data.config === 'string') {
    detail.textContent = data.config;
  } else if (typeof data.error === 'string') {
    detail.textContent = data.error;
  } else {
    detail.textContent = 'No generated source, config, or diff in this response.';
  }
}
function renderDiff(diff) {
  detail.className = 'detail diff';
  const lines = diff.split('\n');
  const raw = lines.map(line => {
    let cls = '';
    if (line.startsWith('+') && !line.startsWith('+++')) cls = 'diff-add';
    else if (line.startsWith('-') && !line.startsWith('---')) cls = 'diff-remove';
    else if (line.startsWith('@@') || line.startsWith('diff ') || line.startsWith('---') || line.startsWith('+++')) cls = 'diff-meta';
    return '<span class="' + cls + '">' + escapeHtml(line || ' ') + '</span>';
  }).join('');
  detail.innerHTML = '<div class="pane-title">Unified Diff</div>' + raw + renderSideBySideDiff(lines);
}
function renderSideBySideDiff(lines) {
  const rows = [];
  for (let index = 0; index < lines.length; index++) {
    const line = lines[index];
    if (!line) continue;
    if (line.startsWith('@@') || line.startsWith('diff ') || line.startsWith('---') || line.startsWith('+++')) {
      rows.push('<div class="side-meta">' + escapeHtml(line) + '</div>');
    } else if (line.startsWith('-')) {
      const next = lines[index + 1] || '';
      if (next.startsWith('+') && !next.startsWith('+++')) {
        rows.push(sideRow(line.slice(1), next.slice(1), 'side-old', 'side-new'));
        index++;
      } else {
        rows.push(sideRow(line.slice(1), '', 'side-old', ''));
      }
    } else if (line.startsWith('+')) {
      rows.push(sideRow('', line.slice(1), '', 'side-new'));
    } else {
      rows.push(sideRow(line.startsWith(' ') ? line.slice(1) : line, line.startsWith(' ') ? line.slice(1) : line, '', ''));
    }
  }
  if (rows.length === 0) return '';
  return '<div class="pane-title">Side-by-Side Diff</div><div class="side-by-side">' + rows.join('') + '</div>';
}
function sideRow(left, right, leftClass, rightClass) {
  return '<div class="side-cell ' + leftClass + '">' + escapeHtml(left || ' ') + '</div><div class="side-cell ' + rightClass + '">' + escapeHtml(right || ' ') + '</div>';
}
function escapeHtml(value) {
  return String(value)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}
function persistFields() {
  const ids = ['dictionary', 'className', 'protocol', 'selector', 'bridgeKey', 'bridgeValue', 'bridgeKeyType', 'bridgeValueType', 'codegenConfig', 'configEditor', 'mappedName', 'mappingClass'];
  for (const id of ids) {
    const element = document.getElementById(id);
    const key = 'gemstone-rs-explorer:' + id;
    try {
      const saved = localStorage.getItem(key);
      if (saved !== null) element.value = saved;
      element.addEventListener('input', () => localStorage.setItem(key, element.value));
      element.addEventListener('change', () => localStorage.setItem(key, element.value));
    } catch {
      return;
    }
  }
}
function clearSavedFields() {
  try {
    for (const key of Object.keys(localStorage)) {
      if (key.startsWith('gemstone-rs-explorer:')) localStorage.removeItem(key);
    }
  } catch {}
  detail.textContent = 'Saved explorer fields cleared. Reload to restore defaults.';
}
function button(label, onClick) {
  const element = document.createElement('button');
  element.className = 'item';
  element.textContent = label;
  element.onclick = onClick;
  return element;
}
async function list(path, key, onPick) {
  const response = await fetch(path);
  const data = await response.json();
  items.innerHTML = '';
  out.textContent = JSON.stringify(data, null, 2);
  for (const value of data[key] || []) items.appendChild(button(value, () => onPick(value)));
}
function loadDictionaries() {
  list('/api/browse/dictionaries', 'dictionaries', value => {
    document.getElementById('dictionary').value = value;
    loadClasses();
  });
}
function loadClasses() {
  list('/api/browse/classes?dictionary=' + q('dictionary'), 'classes', value => {
    document.getElementById('className').value = value;
    loadProtocols();
  });
}
function loadProtocols() {
  list('/api/browse/protocols?class=' + q('className') + '&dictionary=' + q('dictionary'), 'protocols', value => {
    document.getElementById('protocol').value = value;
    loadMethods();
  });
}
function loadMethods() {
  list('/api/browse/methods?class=' + q('className') + '&dictionary=' + q('dictionary') + '&protocol=' + q('protocol'), 'methods', value => {
    document.getElementById('selector').value = value;
    loadSource();
  });
}
function loadSource() {
  callApi('/api/browse/source?class=' + q('className') + '&dictionary=' + q('dictionary') + '&selector=' + q('selector'));
}
async function loadBridgeKeys() {
  const response = await fetch('/api/bridge/keys');
  const data = await response.json();
  items.innerHTML = '';
  out.textContent = JSON.stringify(data, null, 2);
  for (const key of data.keys || []) {
    items.appendChild(button(key.printString, () => {
      document.getElementById('bridgeKey').value = key.printString.replace(/^#/, '');
      getBridgeValue();
    }));
  }
}
function getBridgeValue() { callApi('/api/bridge/get?' + bridgeQuery()); }
function putBridgeValue() {
  callApi('/api/bridge/put?' + bridgeQuery() + '&value=' + q('bridgeValue') + '&value_type=' + q('bridgeValueType'));
}
function removeBridgeValue() { callApi('/api/bridge/remove?' + bridgeQuery()); }
function setConfigEditor(source) {
  const editor = document.getElementById('configEditor');
  editor.value = source;
  try { localStorage.setItem('gemstone-rs-explorer:configEditor', source); } catch {}
}
async function loadCodegenConfig() {
  const data = await callApi('/api/codegen/config?' + codegenQuery());
  if (typeof data.source === 'string') setConfigEditor(data.source);
}
async function saveCodegenConfig() {
  const source = document.getElementById('configEditor').value;
  await callApi('/api/codegen/config/save?' + codegenQuery(), {
    method: 'POST',
    headers: { 'Content-Type': 'text/plain; charset=utf-8' },
    body: source
  });
}
async function codegenSample() {
  const data = await callApi('/api/codegen/sample');
  if (typeof data.config === 'string') setConfigEditor(data.config);
}
async function discoverMappingConfig() {
  const data = await callApi('/api/codegen/discover-mapping?' + codegenQuery() + '&mapped=' + q('mappedName') + '&class=' + q('mappingClass'));
  if (typeof data.config === 'string') setConfigEditor(data.config);
}
function codegenPreview() { callApi('/api/codegen/preview?' + codegenQuery()); }
function codegenDiff() { callApi('/api/codegen/diff?' + codegenQuery()); }
function codegenCheck() { callApi('/api/codegen/check?' + codegenQuery()); }
function codegenGenerate() { callApi('/api/codegen/generate?' + codegenQuery()); }
persistFields();
</script>
</body>
</html>"#,
    );
    html
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
    fn content_length_parses_case_insensitively() {
        let headers = "POST /api/codegen/config/save HTTP/1.1\r\ncontent-length: 42\r\n\r\n";
        assert_eq!(content_length(headers), 42);
        assert_eq!(header_body_offset(headers.as_bytes()), Some(headers.len()));
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
    fn doctor_endpoint_reports_environment() {
        let response = handle_request("GET /api/doctor HTTP/1.1", &ExplorerConfig::default());
        assert!(matches!(response.status, 200 | 503));
        assert!(response.body.contains(r#""environment":"#));
        assert!(response.body.contains(r#""checked":false"#));
    }

    #[test]
    fn landing_page_links_doctor_and_bridge_keys() {
        let response = handle_request("GET / HTTP/1.1", &ExplorerConfig::default());
        assert_eq!(response.status, 200);
        assert!(response.body.contains("gemstone-rs Explorer"));
        assert!(response.body.contains("loadDictionaries()"));
        assert!(response.body.contains("BridgeRoot and Codegen"));
        assert!(response.body.contains("Codegen Workflow"));
        assert!(response.body.contains("codegenConfig"));
        assert!(response.body.contains("configEditor"));
        assert!(response.body.contains("loadCodegenConfig()"));
        assert!(response.body.contains("saveCodegenConfig()"));
        assert!(response.body.contains("method: 'POST'"));
        assert!(response.body.contains("discoverMappingConfig()"));
        assert!(response.body.contains("bridgeKeyType"));
        assert!(response.body.contains("bridgeValueType"));
        assert!(response.body.contains("Generated Source / Config / Diff"));
        assert!(response.body.contains("renderDiff"));
        assert!(response.body.contains("renderSideBySideDiff"));
        assert!(response.body.contains("Side-by-Side Diff"));
        assert!(response.body.contains("localStorage"));
        assert!(response.body.contains("read_only=true"));
        assert!(response.body.contains("allow_eval=false"));
        assert!(response.body.contains("/api/doctor"));
        assert!(response.body.contains("/api/bridge/keys"));
        assert!(response.body.contains("/api/bridge/put"));
        assert!(response.body.contains("/api/codegen/check"));
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
    fn bridge_key_type_query_defaults_to_string() {
        assert_eq!(bridge_key_type_query(Some("symbol")), BridgeKeyType::Symbol);
        assert_eq!(bridge_key_type_query(Some("String")), BridgeKeyType::String);
        assert_eq!(bridge_key_type_query(None), BridgeKeyType::String);
    }

    #[test]
    fn bridge_value_type_query_defaults_to_string() {
        assert_eq!(bridge_value_type_query(None), BridgeValueType::String);
        assert_eq!(
            bridge_value_type_query(Some("SmallInt")),
            BridgeValueType::SmallInt
        );
        assert_eq!(bridge_value_type_query(Some("bool")), BridgeValueType::Bool);
    }

    #[test]
    fn bridge_bool_parser_accepts_common_values() {
        assert!(parse_bridge_bool("true").unwrap());
        assert!(!parse_bridge_bool("0").unwrap());
        assert!(parse_bridge_bool("maybe").is_err());
    }

    #[test]
    fn bridge_put_and_remove_are_disabled_by_default() {
        let put = handle_request(
            "GET /api/bridge/put?key=Demo&value=hello HTTP/1.1",
            &ExplorerConfig::default(),
        );
        assert_eq!(put.status, 403);
        assert!(put.body.contains("allow-write"));

        let remove = handle_request(
            "GET /api/bridge/remove?key=Demo HTTP/1.1",
            &ExplorerConfig::default(),
        );
        assert_eq!(remove.status, 403);
        assert!(remove.body.contains("allow-write"));
    }

    #[test]
    fn bridge_put_validates_key_and_value_before_live_login() {
        let config = ExplorerConfig {
            read_only: false,
            ..ExplorerConfig::default()
        };

        let missing_key = handle_request("GET /api/bridge/put?value=hello HTTP/1.1", &config);
        assert_eq!(missing_key.status, 400);
        assert!(missing_key.body.contains("missing key"));

        let invalid_smallint = handle_request(
            "GET /api/bridge/put?key=Demo&value=nope&value_type=SmallInt HTTP/1.1",
            &config,
        );
        assert_eq!(invalid_smallint.status, 400);
        assert!(invalid_smallint.body.contains("SmallInt"));
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
    fn codegen_config_endpoint_reads_config_text() {
        let config_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/codegen/gemstone-rs.codegen");
        let request = format!(
            "GET /api/codegen/config?config={} HTTP/1.1",
            config_path.display()
        );
        let response = handle_request(&request, &ExplorerConfig::default());
        assert_eq!(response.status, 200);
        assert!(response.body.contains(r#""success":true"#));
        assert!(response.body.contains("gemstone-rs codegen config"));
        assert!(response.body.contains("generated/gemstone_wrappers.rs"));
    }

    #[test]
    fn codegen_config_save_is_disabled_by_default() {
        let response = handle_request(
            "GET /api/codegen/config/save?config=tmp.codegen&source=output%20%3D%20tmp.rs HTTP/1.1",
            &ExplorerConfig::default(),
        );
        assert_eq!(response.status, 403);
        assert!(response.body.contains("allow-write"));
    }

    #[test]
    fn codegen_config_save_validates_before_writing() {
        let config = ExplorerConfig {
            read_only: false,
            ..ExplorerConfig::default()
        };
        let response = handle_request(
            "GET /api/codegen/config/save?config=target/invalid-explorer.codegen&source=not-a-directive HTTP/1.1",
            &config,
        );
        assert_eq!(response.status, 400);
        assert!(response.body.contains("expected key=value"));
    }

    #[test]
    fn post_codegen_config_save_writes_request_body() {
        let path = env::temp_dir().join(format!(
            "gemstone-rs-explorer-post-save-{}.codegen",
            std::process::id()
        ));
        let _ = fs::remove_file(&path);
        let config = ExplorerConfig {
            read_only: false,
            ..ExplorerConfig::default()
        };
        let source = "output = generated/post-save.rs\n";
        let request = HttpRequest {
            method: "POST".to_string(),
            target: format!("/api/codegen/config/save?config={}", path.display()),
            body: source.to_string(),
        };

        let response = handle_http_request(&request, &config);
        assert_eq!(response.status, 200);
        assert!(response.body.contains(r#""success":true"#));
        assert_eq!(fs::read_to_string(&path).unwrap(), source);
        let _ = fs::remove_file(path);
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
