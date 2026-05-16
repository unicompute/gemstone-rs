use gemstone_rs::{
    browser::{Browser, ALL_PROTOCOLS},
    codegen::{self, DEFAULT_CONFIG_PATH},
    gci_library_path, profiles, BridgeKeySummary, BridgeKeyType, BridgeValue, Config, Oop, Session,
    Value, DEFAULT_BRIDGE_ROOT,
};
use std::env;
use std::error::Error as StdError;
use std::fmt;
use std::fs;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
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
    if let Some(path) = config.env_file.as_deref() {
        let loaded = apply_env_file(path)?;
        eprintln!("loaded {loaded} environment values from {}", path.display());
    }
    let listener = TcpListener::bind(config.addr())?;
    eprintln!(
        "gemstone-rs explorer listening at http://{}/",
        config.addr()
    );
    eprintln!(
        "read_only={}, allow_eval={}, auth_required={}",
        config.read_only,
        config.allow_eval,
        config.auth_token.is_some()
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
    let parsed_headers = headers
        .lines()
        .skip(1)
        .filter_map(|line| {
            let (name, value) = line.split_once(':')?;
            Some((name.trim().to_string(), value.trim().to_string()))
        })
        .collect();
    let body = String::from_utf8_lossy(&bytes[body_start..body_start + content_length]).to_string();
    Ok(HttpRequest::from_parts(request_line, parsed_headers, body))
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
    if !request_is_authorized(request, &route, config) {
        return Response::json(
            401,
            r#"{"error":"missing or invalid explorer auth token"}"#.to_string(),
        );
    }

    if request.method != "GET"
        && !(request.method == "POST" && route.path == "/api/codegen/config/save")
        && !(request.method == "POST" && route.path == "/api/codegen/profiles/save")
        && !(request.method == "POST" && route.path == "/api/env/write")
    {
        return Response::json(405, r#"{"error":"method not allowed"}"#.to_string());
    }

    match route.path.as_str() {
        "/" => Response::html(200, landing_html(config)),
        "/health" => Response::json(200, r#"{"status":"ok"}"#.to_string()),
        "/api/config" => Response::json(200, config_json(config)),
        "/api/setup/assistant" => setup_assistant_response(&route, config),
        "/api/doctor" => doctor_response(bool_query(route.query("live").as_deref())),
        "/api/env/sample" => env_sample_response(),
        "/api/env/write" => {
            if config.read_only {
                Response::json(403, r#"{"error":"writes disabled"}"#.to_string())
            } else {
                env_write_response(&route, config)
            }
        }
        "/api/compare/gemstone-py/status" => Response::json(200, gemstone_py_status_json(true)),
        "/api/compare/all/status" => Response::json(200, all_status_json()),
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
        "/api/codegen/config" => codegen_config_response(&route, config),
        "/api/codegen/configs" => codegen_configs_response(&route, config),
        "/api/codegen/profiles" => codegen_profiles_response(&route, config),
        "/api/codegen/profiles/check" => codegen_profiles_check_response(&route, config),
        "/api/codegen/profiles/save" => {
            if config.read_only {
                return Response::json(
                    403,
                    r#"{"error":"codegen profile save disabled; restart with --allow-write"}"#
                        .to_string(),
                );
            }
            codegen_profiles_save_response(&route, config, &request.body)
        }
        "/api/codegen/config/save" => {
            if config.read_only {
                return Response::json(
                    403,
                    r#"{"error":"codegen config save disabled; restart with --allow-write"}"#
                        .to_string(),
                );
            }
            codegen_config_save_response(&route, config, &request.body)
        }
        "/api/codegen/discover-mapping" => codegen_discover_mapping_response(&route, config),
        "/api/codegen/preview" => codegen_preview_response(&route, config),
        "/api/codegen/preview-profile" => codegen_preview_profile_response(&route, config),
        "/api/codegen/explain" => codegen_explain_response(&route, config),
        "/api/codegen/explain-profile" => codegen_explain_profile_response(&route, config),
        "/api/codegen/diff" => codegen_diff_response(&route, config),
        "/api/codegen/diff-profile" => codegen_diff_profile_response(&route, config),
        "/api/codegen/check" => codegen_check_response(&route, config),
        "/api/codegen/check-profile" => codegen_check_profile_response(&route, config),
        "/api/codegen/generate" => {
            if config.read_only {
                return Response::json(
                    403,
                    r#"{"error":"codegen generate disabled; restart with --allow-write"}"#
                        .to_string(),
                );
            }
            codegen_generate_response(&route, config)
        }
        "/api/codegen/generate-profile" => {
            if config.read_only {
                return Response::json(
                    403,
                    r#"{"error":"codegen generate disabled; restart with --allow-write"}"#
                        .to_string(),
                );
            }
            codegen_generate_profile_response(&route, config)
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

fn gemstone_py_status_json(include_view: bool) -> String {
    status_json(StatusJson {
        comparison: "gemstone-py",
        view: include_view.then_some("status"),
        project_label: "gemstone-rs",
        answer: "gemstone-py remains more mature for Python apps, web examples, explorer polish, and release lanes; gemstone-rs is the better fit for Rust-native services, CLIs, typed wrappers, and the future shared native core.",
        gemstone_py_score: 30,
        project_score: 27,
        max_score: 35,
        total_batches: 6,
        hours_min: 40,
        hours_max: 71,
        next_number: 1,
        next_focus: "Explorer and VS Code visual polish",
        next_hours_min: 6,
        next_hours_max: 10,
        next_outcome: "Polish live class browsing, generated-file editing, screenshots, and richer BridgeRoot payload views inside the embedded webview.",
        next_verify_with: "python3 scripts/explorer_endpoint_smoke.py; vscode-gemstone-rs-workbench smoke test",
        top_gap_priority: "P1",
        top_gap_area: "Web framework adapters",
        top_gap_strength: "FastAPI, Litestar, and Django examples are first-class and documented.",
        top_gap_project_gap: "gemstone-rs now has shared JSON health helpers, standard-library HTTP, graceful health-pool startup, route and request-trace headers, packaged Axum/Actix adapters, checked services, route smoke coverage, and installed Axum/Actix scaffolds. It still needs richer framework middleware and request lifecycle examples.",
        top_gap_next_action: "Add richer middleware examples, request lifecycle examples, and live route smoke tests for the packaged Axum/Actix adapters.",
        top_gap_verify_with: "cargo run --manifest-path examples/actix-service/Cargo.toml -- --routes",
        command_target: "gemstone-py",
    })
}

fn gemstone_js_status_json(include_view: bool) -> String {
    status_json(StatusJson {
        comparison: "gemstone-js",
        view: include_view.then_some("status"),
        project_label: "gemstone-js",
        answer: "gemstone-py remains more mature for Python products and visual tooling; gemstone-js is the TypeScript/Node path when async JS services and npm packaging are the product boundary.",
        gemstone_py_score: 32,
        project_score: 21,
        max_score: 35,
        total_batches: 6,
        hours_min: 42,
        hours_max: 72,
        next_number: 1,
        next_focus: "Native publish confidence",
        next_hours_min: 6,
        next_hours_max: 10,
        next_outcome: "Publish and verify gemstone-js plus @gemstone-js/native from a clean install across supported Node platforms.",
        next_verify_with: "cd /Users/tariq/src/gemstone-js && npm run verify && GS_RUN_LIVE=1 npm run test:live",
        top_gap_priority: "P1",
        top_gap_area: "Published native confidence",
        top_gap_strength: "gemstone-py has a working package/release lane and optional native acceleration path.",
        top_gap_project_gap: "gemstone-js is alpha and depends on optional @gemstone-js/native plus Deno/Bun FFI starter paths that need broader live proof.",
        top_gap_next_action: "Publish and verify gemstone-js plus @gemstone-js/native across supported Node platforms, then run npm verify and live smoke tests from a clean install.",
        top_gap_verify_with: "cd /Users/tariq/src/gemstone-js && npm run verify && GS_RUN_LIVE=1 npm run test:live",
        command_target: "gemstone-js",
    })
}

fn all_status_json() -> String {
    format!(
        r#"{{"success":true,"comparison":"all","view":"status","totalBatches":12,"hoursMin":82,"hoursMax":143,"comparisons":[{},{}]}}"#,
        gemstone_py_status_json(false),
        gemstone_js_status_json(false)
    )
}

#[derive(Clone, Copy, Debug)]
struct StatusJson {
    comparison: &'static str,
    view: Option<&'static str>,
    project_label: &'static str,
    answer: &'static str,
    gemstone_py_score: u8,
    project_score: u8,
    max_score: u8,
    total_batches: u8,
    hours_min: u8,
    hours_max: u8,
    next_number: u8,
    next_focus: &'static str,
    next_hours_min: u8,
    next_hours_max: u8,
    next_outcome: &'static str,
    next_verify_with: &'static str,
    top_gap_priority: &'static str,
    top_gap_area: &'static str,
    top_gap_strength: &'static str,
    top_gap_project_gap: &'static str,
    top_gap_next_action: &'static str,
    top_gap_verify_with: &'static str,
    command_target: &'static str,
}

fn status_json(status: StatusJson) -> String {
    let view = status
        .view
        .map(|view| format!(r#","view":"{}""#, escape_json(view)))
        .unwrap_or_default();
    format!(
        r#"{{"success":true,"comparison":"{}"{},"answer":"{}","remaining":{{"totalBatches":{},"hoursMin":{},"hoursMax":{}}},"parity":{{"gemstonePyScore":{},"project":"{}","projectScore":{},"maxScore":{},"scoreGap":{}}},"nextBatch":{{"number":{},"focus":"{}","hoursMin":{},"hoursMax":{},"outcome":"{}","verifyWith":"{}"}},"topGap":{{"priority":"{}","area":"{}","gemstonePyStrength":"{}","project":"{}","projectGap":"{}","nextAction":"{}","verifyWith":"{}"}},"commands":{{"status":"gemstone-rs compare {} --status","scorecard":"gemstone-rs compare {} --scorecard","parity":"gemstone-rs compare {} --parity","batches":"gemstone-rs compare {} --batches","totals":"gemstone-rs compare {} --totals"}}}}"#,
        escape_json(status.comparison),
        view,
        escape_json(status.answer),
        status.total_batches,
        status.hours_min,
        status.hours_max,
        status.gemstone_py_score,
        escape_json(status.project_label),
        status.project_score,
        status.max_score,
        status
            .gemstone_py_score
            .saturating_sub(status.project_score),
        status.next_number,
        escape_json(status.next_focus),
        status.next_hours_min,
        status.next_hours_max,
        escape_json(status.next_outcome),
        escape_json(status.next_verify_with),
        escape_json(status.top_gap_priority),
        escape_json(status.top_gap_area),
        escape_json(status.top_gap_strength),
        escape_json(status.project_label),
        escape_json(status.top_gap_project_gap),
        escape_json(status.top_gap_next_action),
        escape_json(status.top_gap_verify_with),
        escape_json(status.command_target),
        escape_json(status.command_target),
        escape_json(status.command_target),
        escape_json(status.command_target),
        escape_json(status.command_target)
    )
}

fn setup_assistant_response(route: &Route, config: &ExplorerConfig) -> Response {
    let env_path = match codegen_write_path(route, config, "env_file", ".env.gemstone-rs") {
        Ok(path) => path,
        Err(err) => return Response::json(400, format!(r#"{{"error":"{}"}}"#, escape_json(&err))),
    };
    let config_path = codegen_config_path(route, config);
    let profile_path = codegen_profile_path(route, config);
    let mut steps = Vec::new();

    steps.push(setup_step_json(
        "Environment file",
        env_path.exists(),
        format!("{} exists", env_path.display()),
        if env_path.exists() {
            None
        } else {
            Some("Click Write .env.gemstone-rs, edit placeholders, then restart with --env-file.")
        },
    ));

    match Config::from_env() {
        Ok(config) => {
            steps.push(setup_step_json(
                "GemStone environment",
                true,
                format!(
                    "stone={} host={} username={}",
                    config.stone, config.host, config.username
                ),
                None,
            ));
            match gci_library_path(&config) {
                Ok(path) => steps.push(setup_step_json(
                    "GCI library",
                    true,
                    path.display().to_string(),
                    None,
                )),
                Err(err) => steps.push(setup_step_json(
                    "GCI library",
                    false,
                    err.to_string(),
                    Some("Set GS_LIB, GS_LIB_PATH, or GEMSTONE in the env file."),
                )),
            }
        }
        Err(err) => steps.push(setup_step_json(
            "GemStone environment",
            false,
            err.to_string(),
            Some("Set GS_USERNAME, GS_PASSWORD, GS_STONE_NAME, and connection values."),
        )),
    }

    match codegen::Config::from_file(&config_path) {
        Ok(codegen_config) => steps.push(setup_step_json(
            "Codegen config",
            true,
            format!(
                "{} classes={} mapped={}",
                config_path.display(),
                codegen_config.classes.len(),
                codegen_config.mapped.len()
            ),
            None,
        )),
        Err(err) => steps.push(setup_step_json(
            "Codegen config",
            false,
            err.to_string(),
            Some("Load or create a gemstone-rs.codegen file, then run Explain."),
        )),
    }

    match profiles::validate_file(&profile_path) {
        Ok(report) => steps.push(setup_step_json(
            "Project profiles",
            true,
            format!(
                "{} profiles={}",
                profile_path.display(),
                report.profile_count
            ),
            None,
        )),
        Err(err) => steps.push(setup_step_json(
            "Project profiles",
            false,
            err.to_string(),
            Some("Create or load gemstone-rs.codegen-profiles.json."),
        )),
    }

    let success = !steps.iter().any(|step| step.contains(r#""ok":false"#));
    Response::json(
        if success { 200 } else { 503 },
        format!(r#"{{"success":{},"steps":[{}]}}"#, success, steps.join(",")),
    )
}

fn setup_step_json(name: &str, ok: bool, detail: impl AsRef<str>, action: Option<&str>) -> String {
    format!(
        r#"{{"name":"{}","ok":{},"detail":"{}","action":{}}}"#,
        escape_json(name),
        ok,
        escape_json(detail.as_ref()),
        optional_json_string(action)
    )
}

fn env_sample_response() -> Response {
    Response::json(
        200,
        format!(
            r#"{{"success":true,"source":"{}"}}"#,
            escape_json(&env_template_source())
        ),
    )
}

fn env_write_response(route: &Route, config: &ExplorerConfig) -> Response {
    let path = match codegen_write_path(route, config, "path", ".env.gemstone-rs") {
        Ok(path) => path,
        Err(err) => return Response::json(400, format!(r#"{{"error":"{}"}}"#, escape_json(&err))),
    };
    if let Some(parent) = path.parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            return Response::json(
                500,
                format!(
                    r#"{{"success":false,"path":"{}","error":"{}"}}"#,
                    escape_json(&path.display().to_string()),
                    escape_json(&err.to_string())
                ),
            );
        }
    }
    let source = env_template_source();
    match fs::write(&path, source) {
        Ok(()) => Response::json(
            200,
            format!(
                r#"{{"success":true,"path":"{}"}}"#,
                escape_json(&path.display().to_string())
            ),
        ),
        Err(err) => Response::json(
            500,
            format!(
                r#"{{"success":false,"path":"{}","error":"{}"}}"#,
                escape_json(&path.display().to_string()),
                escape_json(&err.to_string())
            ),
        ),
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

fn env_template_source() -> String {
    let stone = env::var("GS_STONE")
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| {
            env::var("GS_STONE_NAME")
                .ok()
                .filter(|value| !value.is_empty())
        })
        .unwrap_or_else(|| "gs64stone".to_string());
    let stone_alias = env::var("GS_STONE_NAME")
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| stone.clone());
    let mut output = String::new();
    output.push_str("# gemstone-rs GemStone/S environment template\n");
    output.push_str(
        "# Password values are placeholders and are never copied from the current environment.\n\n",
    );
    append_env_export(
        &mut output,
        "GS_LIB",
        &env::var("GS_LIB").unwrap_or_else(|_| "/opt/gemstone/product/lib".to_string()),
    );
    output.push_str("# export GS_LIB_PATH='/full/path/to/libgcirpc.dylib'\n");
    append_env_export(&mut output, "GS_STONE", &stone);
    append_env_export(&mut output, "GS_STONE_NAME", &stone_alias);
    append_env_export(
        &mut output,
        "GS_HOST",
        &env::var("GS_HOST").unwrap_or_else(|_| "localhost".to_string()),
    );
    append_env_export(
        &mut output,
        "GS_NETLDI",
        &env::var("GS_NETLDI").unwrap_or_else(|_| "netldi".to_string()),
    );
    append_env_export(
        &mut output,
        "GS_GEM_SERVICE",
        &env::var("GS_GEM_SERVICE").unwrap_or_else(|_| "gemnetobject".to_string()),
    );
    append_env_export(
        &mut output,
        "GS_USERNAME",
        &env::var("GS_USERNAME").unwrap_or_else(|_| "DataCurator".to_string()),
    );
    append_env_export(&mut output, "GS_PASSWORD", "<set-your-password>");
    output.push_str("# export GS_HOST_USERNAME='host-user-if-needed'\n");
    output.push_str("# export GS_HOST_PASSWORD='<set-host-password-if-needed>'\n");
    output.push_str("\n# Verify after editing:\n# gemstone-rs doctor --env-file .env.gemstone-rs\n# gemstone-rs doctor --env-file .env.gemstone-rs --live\n");
    output
}

fn apply_env_file(path: &Path) -> Result<usize, ExplorerError> {
    let source = fs::read_to_string(path)?;
    let values = parse_env_file_source(&source, path)?;
    let count = values.len();
    for (name, value) in values {
        env::set_var(name, value);
    }
    Ok(count)
}

fn parse_env_file_source(
    source: &str,
    path: &Path,
) -> Result<Vec<(String, String)>, ExplorerError> {
    let mut values = Vec::new();
    for (index, raw_line) in source.lines().enumerate() {
        if let Some((name, value)) = parse_env_file_line(raw_line, path, index + 1)? {
            values.push((name, value));
        }
    }
    Ok(values)
}

fn parse_env_file_line(
    raw_line: &str,
    path: &Path,
    line_no: usize,
) -> Result<Option<(String, String)>, ExplorerError> {
    let mut line = raw_line.trim();
    if line.is_empty() || line.starts_with('#') {
        return Ok(None);
    }
    if let Some(rest) = line.strip_prefix("export ") {
        line = rest.trim_start();
    }
    let Some((name, raw_value)) = line.split_once('=') else {
        return Err(env_file_error(path, line_no, "expected NAME=value"));
    };
    let name = name.trim();
    validate_env_file_name(name).map_err(|message| env_file_error(path, line_no, message))?;
    let value = parse_env_file_value(raw_value)
        .map_err(|message| env_file_error(path, line_no, message))?;
    if is_placeholder_env_value(name, &value) {
        return Ok(None);
    }
    Ok(Some((name.to_string(), value)))
}

fn validate_env_file_name(name: &str) -> Result<(), &'static str> {
    if name != "GEMSTONE" && !name.starts_with("GS_") {
        return Err("only GEMSTONE and GS_* variables are allowed in --env-file");
    }
    if name.is_empty()
        || !name
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
    {
        return Err("invalid environment variable name");
    }
    Ok(())
}

fn parse_env_file_value(raw_value: &str) -> Result<String, &'static str> {
    let mut value = String::new();
    let mut chars = raw_value.trim_start().chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\'' => loop {
                match chars.next() {
                    Some('\'') => break,
                    Some(ch) => value.push(ch),
                    None => return Err("unterminated single-quoted value"),
                }
            },
            '"' => loop {
                match chars.next() {
                    Some('"') => break,
                    Some('\\') => {
                        if let Some(ch) = chars.next() {
                            value.push(ch);
                        }
                    }
                    Some(ch) => value.push(ch),
                    None => return Err("unterminated double-quoted value"),
                }
            },
            '\\' => {
                if let Some(ch) = chars.next() {
                    value.push(ch);
                }
            }
            '#' if value.chars().last().is_none_or(char::is_whitespace) => break,
            ch if ch.is_whitespace() => {
                while matches!(chars.peek(), Some(next) if next.is_whitespace()) {
                    chars.next();
                }
                if matches!(chars.peek(), Some('#')) {
                    break;
                }
                value.push(ch);
            }
            ch => value.push(ch),
        }
    }
    Ok(value.trim_end().to_string())
}

fn is_placeholder_env_value(name: &str, value: &str) -> bool {
    matches!(
        (name, value),
        ("GS_PASSWORD", "<set-your-password>")
            | ("GS_HOST_PASSWORD", "<set-host-password-if-needed>")
            | ("GS_LIB_PATH", "/full/path/to/libgcirpc.dylib")
    )
}

fn env_file_error(path: &Path, line_no: usize, message: impl Into<String>) -> ExplorerError {
    ExplorerError::usage(format!("{}:{line_no}: {}", path.display(), message.into()))
}

fn append_env_export(output: &mut String, name: &str, value: &str) {
    output.push_str("export ");
    output.push_str(name);
    output.push('=');
    output.push_str(&shell_quote(value));
    output.push('\n');
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn codegen_root_path(route: &Route, config: &ExplorerConfig) -> PathBuf {
    route
        .non_empty_query("root")
        .map(PathBuf::from)
        .unwrap_or_else(|| config.codegen_root.clone())
}

fn codegen_config_path(route: &Route, config: &ExplorerConfig) -> PathBuf {
    let path = route
        .non_empty_query("config")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_PATH));
    if path.is_absolute() {
        path
    } else {
        codegen_root_path(route, config).join(path)
    }
}

fn codegen_profile_path(route: &Route, config: &ExplorerConfig) -> PathBuf {
    let path = route
        .non_empty_query("profile_file")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(profiles::DEFAULT_PROFILE_PATH));
    if path.is_absolute() {
        path
    } else {
        codegen_root_path(route, config).join(path)
    }
}

fn codegen_write_path(
    route: &Route,
    config: &ExplorerConfig,
    query_name: &str,
    default_path: &str,
) -> Result<PathBuf, String> {
    let path = route
        .non_empty_query(query_name)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(default_path));
    validate_write_component(&path, query_name, config.allow_absolute_write_paths)?;
    if path.is_absolute() {
        return Ok(path);
    }

    let root = if let Some(root) = route.non_empty_query("root").map(PathBuf::from) {
        validate_write_component(&root, "root", config.allow_absolute_write_paths)?;
        if root.is_absolute() {
            root
        } else {
            config.codegen_root.join(root)
        }
    } else {
        validate_no_parent_component(&config.codegen_root, "codegen_root")?;
        config.codegen_root.clone()
    };
    Ok(root.join(path))
}

fn validate_write_component(path: &Path, field: &str, allow_absolute: bool) -> Result<(), String> {
    if path.is_absolute() && !allow_absolute {
        return Err(format!(
            "{field} absolute paths are disabled; restart with --allow-absolute-write-paths"
        ));
    }
    for component in path.components() {
        match component {
            Component::ParentDir => {
                return Err(format!("{field} must not contain '..' path traversal"));
            }
            Component::Prefix(_) | Component::RootDir if !allow_absolute => {
                return Err(format!("{field} absolute paths are disabled"));
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_no_parent_component(path: &Path, field: &str) -> Result<(), String> {
    for component in path.components() {
        if matches!(component, Component::ParentDir) {
            return Err(format!("{field} must not contain '..' path traversal"));
        }
    }
    Ok(())
}

fn codegen_config_response(route: &Route, config: &ExplorerConfig) -> Response {
    let path = codegen_config_path(route, config);
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

fn codegen_profiles_response(route: &Route, config: &ExplorerConfig) -> Response {
    let path = codegen_profile_path(route, config);
    match fs::read_to_string(&path) {
        Ok(source) => Response::json(
            200,
            format!(
                r#"{{"success":true,"profileFile":"{}","source":"{}"}}"#,
                escape_json(&path.display().to_string()),
                escape_json(&source)
            ),
        ),
        Err(err) => Response::json(
            404,
            format!(
                r#"{{"success":false,"profileFile":"{}","error":"{}"}}"#,
                escape_json(&path.display().to_string()),
                escape_json(&err.to_string())
            ),
        ),
    }
}

fn codegen_profiles_save_response(route: &Route, config: &ExplorerConfig, body: &str) -> Response {
    let path = match codegen_write_path(
        route,
        config,
        "profile_file",
        profiles::DEFAULT_PROFILE_PATH,
    ) {
        Ok(path) => path,
        Err(message) => {
            return Response::json(400, format!(r#"{{"error":"{}"}}"#, escape_json(&message)))
        }
    };
    if body.trim().is_empty() {
        return Response::json(400, r#"{"error":"missing profile source"}"#.to_string());
    }
    if let Err(message) = profiles::validate_source(body) {
        return Response::json(
            400,
            format!(
                r#"{{"success":false,"profileFile":"{}","error":"{}"}}"#,
                escape_json(&path.display().to_string()),
                escape_json(&message.to_string())
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
                    r#"{{"success":false,"profileFile":"{}","error":"{}"}}"#,
                    escape_json(&path.display().to_string()),
                    escape_json(&err.to_string())
                ),
            );
        }
    }
    match fs::write(&path, body) {
        Ok(()) => Response::json(
            200,
            format!(
                r#"{{"success":true,"profileFile":"{}","bytes":{},"source":"{}"}}"#,
                escape_json(&path.display().to_string()),
                body.len(),
                escape_json(body)
            ),
        ),
        Err(err) => Response::json(
            500,
            format!(
                r#"{{"success":false,"profileFile":"{}","error":"{}"}}"#,
                escape_json(&path.display().to_string()),
                escape_json(&err.to_string())
            ),
        ),
    }
}

fn codegen_configs_response(route: &Route, config: &ExplorerConfig) -> Response {
    let root = codegen_root_path(route, config);
    if !root.is_dir() {
        return Response::json(
            400,
            format!(
                r#"{{"success":false,"root":"{}","error":"root is not a directory"}}"#,
                escape_json(&root.display().to_string())
            ),
        );
    }

    let mut paths = Vec::new();
    if let Err(err) = collect_codegen_configs(&root, 0, &mut paths) {
        return Response::json(
            500,
            format!(
                r#"{{"success":false,"root":"{}","error":"{}"}}"#,
                escape_json(&root.display().to_string()),
                escape_json(&err.to_string())
            ),
        );
    }

    paths.sort();
    paths.truncate(200);
    let configs = paths
        .iter()
        .map(|path| {
            path.strip_prefix(&root)
                .unwrap_or(path)
                .display()
                .to_string()
        })
        .collect::<Vec<_>>();
    Response::json(
        200,
        format!(
            r#"{{"success":true,"root":"{}","configs":{}}}"#,
            escape_json(&root.display().to_string()),
            json_string_array(configs)
        ),
    )
}

fn collect_codegen_configs(
    root: &Path,
    depth: usize,
    paths: &mut Vec<PathBuf>,
) -> std::io::Result<()> {
    if depth > 6 || paths.len() >= 200 {
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if matches!(
                name.as_ref(),
                ".git"
                    | ".vscode-test"
                    | "target"
                    | "node_modules"
                    | ".pdf-build"
                    | "pdf"
                    | "__pycache__"
            ) {
                continue;
            }
            collect_codegen_configs(&path, depth + 1, paths)?;
        } else if file_type.is_file()
            && path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension == "codegen")
        {
            paths.push(path);
        }
        if paths.len() >= 200 {
            break;
        }
    }
    Ok(())
}

fn codegen_config_save_response(route: &Route, config: &ExplorerConfig, body: &str) -> Response {
    let path = match codegen_write_path(route, config, "config", DEFAULT_CONFIG_PATH) {
        Ok(path) => path,
        Err(message) => {
            return Response::json(400, format!(r#"{{"error":"{}"}}"#, escape_json(&message)))
        }
    };
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

struct CodegenProfileTarget {
    profile_name: String,
    profile_file: PathBuf,
    config_path: PathBuf,
}

fn codegen_profile_target(
    route: &Route,
    config: &ExplorerConfig,
) -> Result<CodegenProfileTarget, Response> {
    let profile_name = route
        .non_empty_query("profile")
        .unwrap_or_else(|| "default".to_string());
    let profile_file = codegen_profile_path(route, config);
    let project = profiles::load_file(&profile_file).map_err(|err| {
        Response::json(
            500,
            format!(
                r#"{{"success":false,"profileFile":"{}","error":"{}"}}"#,
                escape_json(&profile_file.display().to_string()),
                escape_json(&err.to_string())
            ),
        )
    })?;
    let Some(profile) = project.get(&profile_name) else {
        return Err(Response::json(
            404,
            format!(
                r#"{{"success":false,"profile":"{}","profileFile":"{}","error":"profile not found"}}"#,
                escape_json(&profile_name),
                escape_json(&profile_file.display().to_string())
            ),
        ));
    };
    let base_root = codegen_root_path(route, config);
    let config_path = match profile.resolved_config_path_with_base_root(Some(&base_root)) {
        Ok(path) => path,
        Err(err) => {
            return Err(Response::json(
                400,
                format!(
                    r#"{{"success":false,"profile":"{}","profileFile":"{}","error":"{}"}}"#,
                    escape_json(&profile_name),
                    escape_json(&profile_file.display().to_string()),
                    escape_json(&err.to_string())
                ),
            ))
        }
    };

    Ok(CodegenProfileTarget {
        profile_name,
        profile_file,
        config_path,
    })
}

fn codegen_profiles_check_response(route: &Route, config: &ExplorerConfig) -> Response {
    let profile_file = codegen_profile_path(route, config);
    let base_root = codegen_root_path(route, config);
    match profiles::check_file(&profile_file, Some(&base_root)) {
        Ok(report) => Response::json(200, report.to_json()),
        Err(err) => Response::json(
            if profile_file.exists() { 400 } else { 404 },
            format!(
                r#"{{"success":false,"profileFile":"{}","error":"{}"}}"#,
                escape_json(&profile_file.display().to_string()),
                escape_json(&err.to_string())
            ),
        ),
    }
}

fn codegen_preview_response(route: &Route, config: &ExplorerConfig) -> Response {
    let path = codegen_config_path(route, config);
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

fn codegen_preview_profile_response(route: &Route, config: &ExplorerConfig) -> Response {
    let target = match codegen_profile_target(route, config) {
        Ok(target) => target,
        Err(response) => return response,
    };
    match codegen::Config::from_file(&target.config_path).map(|config| codegen::generate(&config)) {
        Ok(generated) => Response::json(
            200,
            format!(
                r#"{{"success":true,"profile":"{}","profileFile":"{}","config":"{}","output":"{}","source":"{}"}}"#,
                escape_json(&target.profile_name),
                escape_json(&target.profile_file.display().to_string()),
                escape_json(&target.config_path.display().to_string()),
                escape_json(&generated.output.display().to_string()),
                escape_json(&generated.source)
            ),
        ),
        Err(err) => codegen_error_response(err),
    }
}

fn codegen_explain_response(route: &Route, config: &ExplorerConfig) -> Response {
    let path = codegen_config_path(route, config);
    match codegen::Config::from_file(&path) {
        Ok(config) => Response::json(
            200,
            format!(
                r#"{{"success":true,"config":"{}","explain":{}}}"#,
                escape_json(&path.display().to_string()),
                codegen::explain_json(&config)
            ),
        ),
        Err(err) => codegen_error_response(err),
    }
}

fn codegen_explain_profile_response(route: &Route, config: &ExplorerConfig) -> Response {
    let target = match codegen_profile_target(route, config) {
        Ok(target) => target,
        Err(response) => return response,
    };
    match codegen::Config::from_file(&target.config_path) {
        Ok(config) => Response::json(
            200,
            format!(
                r#"{{"success":true,"profile":"{}","profileFile":"{}","config":"{}","explain":{}}}"#,
                escape_json(&target.profile_name),
                escape_json(&target.profile_file.display().to_string()),
                escape_json(&target.config_path.display().to_string()),
                codegen::explain_json(&config)
            ),
        ),
        Err(err) => codegen_error_response(err),
    }
}

fn codegen_check_profile_response(route: &Route, config: &ExplorerConfig) -> Response {
    let target = match codegen_profile_target(route, config) {
        Ok(target) => target,
        Err(response) => return response,
    };
    match codegen::Config::from_file(&target.config_path).and_then(|config| codegen::check(&config))
    {
        Ok(report) => Response::json(
            200,
            format!(
                r#"{{"success":true,"profile":"{}","profileFile":"{}","config":"{}","output":"{}","exists":{},"upToDate":{}}}"#,
                escape_json(&target.profile_name),
                escape_json(&target.profile_file.display().to_string()),
                escape_json(&target.config_path.display().to_string()),
                escape_json(&report.output.display().to_string()),
                report.exists,
                report.up_to_date
            ),
        ),
        Err(err) => codegen_error_response(err),
    }
}

fn codegen_check_response(route: &Route, config: &ExplorerConfig) -> Response {
    let path = codegen_config_path(route, config);
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

fn codegen_diff_profile_response(route: &Route, config: &ExplorerConfig) -> Response {
    let target = match codegen_profile_target(route, config) {
        Ok(target) => target,
        Err(response) => return response,
    };
    match codegen::Config::from_file(&target.config_path).and_then(|config| codegen::diff(&config))
    {
        Ok(report) => Response::json(
            200,
            format!(
                r#"{{"success":true,"profile":"{}","profileFile":"{}","config":"{}","output":"{}","exists":{},"upToDate":{},"diff":"{}"}}"#,
                escape_json(&target.profile_name),
                escape_json(&target.profile_file.display().to_string()),
                escape_json(&target.config_path.display().to_string()),
                escape_json(&report.output.display().to_string()),
                report.exists,
                report.up_to_date,
                escape_json(&report.diff)
            ),
        ),
        Err(err) => codegen_error_response(err),
    }
}

fn codegen_diff_response(route: &Route, config: &ExplorerConfig) -> Response {
    let path = codegen_config_path(route, config);
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

fn codegen_generate_profile_response(route: &Route, config: &ExplorerConfig) -> Response {
    let target = match codegen_profile_target(route, config) {
        Ok(target) => target,
        Err(response) => return response,
    };
    match codegen::Config::from_file(&target.config_path)
        .and_then(|config| codegen::generate_to_file(&config))
    {
        Ok(generated) => Response::json(
            200,
            format!(
                r#"{{"success":true,"profile":"{}","profileFile":"{}","config":"{}","output":"{}","bytes":{}}}"#,
                escape_json(&target.profile_name),
                escape_json(&target.profile_file.display().to_string()),
                escape_json(&target.config_path.display().to_string()),
                escape_json(&generated.output.display().to_string()),
                generated.source.len()
            ),
        ),
        Err(err) => codegen_error_response(err),
    }
}

fn codegen_generate_response(route: &Route, config: &ExplorerConfig) -> Response {
    let path = codegen_config_path(route, config);
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

fn codegen_discover_mapping_response(route: &Route, config: &ExplorerConfig) -> Response {
    let path = codegen_config_path(route, config);
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
    Symbol,
    SmallInt,
    Bool,
}

impl BridgeValueType {
    fn name(self) -> &'static str {
        match self {
            Self::String => "String",
            Self::Symbol => "Symbol",
            Self::SmallInt => "SmallInt",
            Self::Bool => "Bool",
        }
    }

    fn parse_value(self, value: &str) -> Result<BridgeValue, String> {
        match self {
            Self::String => Ok(BridgeValue::from(value.to_string())),
            Self::Symbol => Ok(BridgeValue::Symbol(value.to_string())),
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
        Some("symbol" | "sym") => BridgeValueType::Symbol,
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

fn optional_json_string(value: Option<&str>) -> String {
    value
        .map(|value| format!(r#""{}""#, escape_json(value)))
        .unwrap_or_else(|| "null".to_string())
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
    codegen_root: PathBuf,
    allow_absolute_write_paths: bool,
    env_file: Option<PathBuf>,
    auth_token: Option<String>,
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
                "--codegen-root" => {
                    index += 1;
                    let value = args
                        .get(index)
                        .ok_or_else(|| ExplorerError::usage("missing value for --codegen-root"))?;
                    config.codegen_root = PathBuf::from(value);
                }
                "--env-file" => {
                    index += 1;
                    let value = args
                        .get(index)
                        .ok_or_else(|| ExplorerError::usage("missing value for --env-file"))?;
                    config.env_file = Some(PathBuf::from(value));
                }
                option if option.starts_with("--env-file=") => {
                    let (_, value) = option.split_once('=').unwrap_or_default();
                    if value.is_empty() {
                        return Err(ExplorerError::usage("missing value for --env-file="));
                    }
                    config.env_file = Some(PathBuf::from(value));
                }
                "--auth-token" => {
                    index += 1;
                    let value = args
                        .get(index)
                        .ok_or_else(|| ExplorerError::usage("missing value for --auth-token"))?;
                    config.auth_token = Some(non_empty_auth_token(value)?);
                }
                option if option.starts_with("--auth-token=") => {
                    let (_, value) = option.split_once('=').unwrap_or_default();
                    config.auth_token = Some(non_empty_auth_token(value)?);
                }
                "--auth-token-env" => {
                    index += 1;
                    let name = args.get(index).ok_or_else(|| {
                        ExplorerError::usage("missing value for --auth-token-env")
                    })?;
                    let value = env::var(name).map_err(|_| {
                        ExplorerError::usage(format!("environment variable {name} is not set"))
                    })?;
                    config.auth_token = Some(non_empty_auth_token(&value)?);
                }
                option if option.starts_with("--auth-token-env=") => {
                    let (_, name) = option.split_once('=').unwrap_or_default();
                    if name.is_empty() {
                        return Err(ExplorerError::usage("missing value for --auth-token-env="));
                    }
                    let value = env::var(name).map_err(|_| {
                        ExplorerError::usage(format!("environment variable {name} is not set"))
                    })?;
                    config.auth_token = Some(non_empty_auth_token(&value)?);
                }
                "--allow-eval" => config.allow_eval = true,
                "--allow-write" => config.read_only = false,
                "--allow-absolute-write-paths" => config.allow_absolute_write_paths = true,
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
            codegen_root: PathBuf::from("."),
            allow_absolute_write_paths: false,
            env_file: None,
            auth_token: None,
        }
    }
}

fn non_empty_auth_token(value: &str) -> Result<String, ExplorerError> {
    let token = value.trim();
    if token.is_empty() {
        return Err(ExplorerError::usage(
            "explorer auth token must not be empty",
        ));
    }
    Ok(token.to_string())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct HttpRequest {
    method: String,
    target: String,
    headers: Vec<(String, String)>,
    body: String,
}

impl HttpRequest {
    #[cfg(test)]
    fn from_request_line(request_line: &str, body: String) -> Option<Self> {
        Self::from_parts(request_line, Vec::new(), body)
    }

    fn from_parts(
        request_line: &str,
        headers: Vec<(String, String)>,
        body: String,
    ) -> Option<Self> {
        let mut parts = request_line.split_whitespace();
        let method = parts.next()?.to_string();
        let target = parts.next()?.to_string();
        Some(Self {
            method,
            target,
            headers,
            body,
        })
    }

    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
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
    let auth_note = if config.auth_token.is_some() {
        "Auth token required."
    } else {
        "Auth token disabled."
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
textarea.compact, pre.compact { min-height: 80px; }
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
.profile-check { white-space: normal; }
.profile-table { width: 100%; border-collapse: collapse; color: #e6edf3; }
.profile-table th, .profile-table td { border-bottom: 1px solid #30363d; padding: 6px; text-align: left; vertical-align: top; }
.profile-table button { padding: 4px 6px; margin: 0 4px 4px 0; }
.profile-ok { color: #7ee787; }
.profile-stale { color: #ffa657; }
.profile-error { color: #ff7b72; }
.compare-card { display: grid; gap: 6px; border: 1px solid #30363d; border-radius: 8px; padding: 10px; margin: 8px 0; background: #161b22; color: #e6edf3; }
.compare-card h3 { margin: 0; color: #f0f6fc; font-size: 14px; }
.compare-card p { margin: 0; }
.compare-score { display: flex; flex-wrap: wrap; gap: 8px; }
.compare-score span { border: 1px solid #30363d; border-radius: 999px; padding: 3px 7px; color: #c9d1d9; }
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
        r#"<span class="pill">read_only={}</span><span class="pill">allow_eval={}</span><span class="pill">auth_required={}</span><span class="pill">{}</span><span class="pill">{}</span><span class="pill">{}</span>"#,
        config.read_only,
        config.allow_eval,
        config.auth_token.is_some(),
        write_note,
        eval_note,
        auth_note
    ));
    html.push_str(
        r#"
</div>
</header>
<main>
<section id="browse">
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
<section id="bridge-codegen">
<h2>BridgeRoot and Codegen</h2>
<div class="actions">
<button onclick="runSetupAssistant()">Run Setup Assistant</button>
<button onclick="callApi('/api/doctor?live=1')">Doctor Live</button>
<button class="secondary" onclick="showEnvTemplate()">Show Env Template</button>
<button class="secondary" onclick="writeEnvTemplate()">Write .env.gemstone-rs</button>
<button onclick="callApi('/api/status')">Status</button>
<button onclick="callApi('/api/bridge/root')">BridgeRoot</button>
<button onclick="loadBridgeKeys()">Keys</button>
</div>
<h2>Comparison Status</h2>
<div class="actions">
<button class="secondary" onclick="compareGemstonePy()">Compare with gemstone-py</button>
<button class="secondary" onclick="compareAll()">Show All Comparison Status</button>
</div>
<label>Env file path<input id="envFilePath" value=".env.gemstone-rs"></label>
<div class="row">
<label>Bridge key<input id="bridgeKey" value="WorkbenchDraft"></label>
<label>Bridge value<input id="bridgeValue" value="hello"></label>
</div>
<div class="row">
<label>Bridge key type<select id="bridgeKeyType"><option>String</option><option>Symbol</option></select></label>
<label>Bridge value type<select id="bridgeValueType"><option>String</option><option>Symbol</option><option>SmallInt</option><option>Bool</option></select></label>
</div>
<div class="actions">
<button class="secondary" onclick="getBridgeValue()">Get</button>
<button class="secondary" onclick="putBridgeValue()">Put Value</button>
<button class="secondary" onclick="removeBridgeValue()">Remove</button>
</div>
<h2 id="codegen-workflow">Codegen Workflow</h2>
<label>Config path<input id="codegenConfig" value="examples/codegen/gemstone-rs.codegen"></label>
<label>Config root<input id="codegenRoot" placeholder="server default from --codegen-root"></label>
<label>Known configs<select id="codegenConfigPicker" onchange="pickCodegenConfig()"><option value="">Refresh to list configs</option></select></label>
<label>Recent configs<select id="codegenRecent" onchange="pickRecentCodegenConfig()"><option value="">No recent configs</option></select></label>
<div class="row">
<label>Profile name<input id="profileName" value="default"></label>
<label>Saved profiles<select id="codegenProfile" onchange="loadCodegenProfile()"><option value="">No saved profiles</option></select></label>
</div>
<label>Profile JSON<textarea id="profileJson" class="compact" spellcheck="false">Export a profile here, or paste one to import.</textarea></label>
<label>Project profile file<input id="profileFile" value="gemstone-rs.codegen-profiles.json"></label>
<div class="pane-title">Import Summary</div>
<pre id="importSummary" class="compact">No profile import yet.</pre>
<label>Config editor<textarea id="configEditor" spellcheck="false">Load a config, sample config, or discovered mapping proposal.</textarea></label>
<div class="row">
<label>Mapped Rust type<input id="mappedName" value="BookingDraft"></label>
<label>GemStone class<input id="mappingClass" value="Object"></label>
</div>
<div class="actions">
<button class="secondary" onclick="loadCodegenConfigs()">Refresh Configs</button>
<button class="secondary" onclick="loadCodegenConfig()">Load Config</button>
<button class="secondary" onclick="saveCodegenConfig()">Save Config</button>
<button class="secondary" onclick="codegenSample()">Sample Config</button>
<button class="secondary" onclick="discoverMappingConfig()">Discover Mapping</button>
<button class="secondary" onclick="codegenExplain()">Explain</button>
<button class="secondary" onclick="codegenExplainProfile()">Explain Profile</button>
<button class="secondary" onclick="codegenPreview()">Preview</button>
<button class="secondary" onclick="codegenPreviewProfile()">Preview Profile</button>
<button class="secondary" onclick="codegenDiff()">Diff</button>
<button class="secondary" onclick="codegenDiffProfile()">Diff Profile</button>
<button class="secondary" onclick="codegenCheck()">Check</button>
<button class="secondary" onclick="codegenCheckProfile()">Check Profile</button>
<button class="secondary" onclick="codegenGenerate()">Generate</button>
<button class="secondary" onclick="codegenGenerateProfile()">Generate Profile</button>
<button class="secondary" onclick="saveCodegenProfile()">Save Profile</button>
<button class="secondary" onclick="exportCodegenProfile()">Export Profile</button>
<button class="secondary" onclick="importCodegenProfile()">Import Profile</button>
<button class="secondary" onclick="loadProjectProfiles()">Load Project Profiles</button>
<button class="secondary" onclick="checkProjectProfiles()">Check Project Profiles</button>
<button class="secondary" onclick="saveProjectProfiles()">Save Project Profiles</button>
<button class="secondary" onclick="deleteCodegenProfile()">Delete Profile</button>
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
const importSummary = document.getElementById('importSummary');
const recentConfigsKey = 'gemstone-rs-explorer:recentCodegenConfigs';
const profilesKey = 'gemstone-rs-explorer:codegenProfiles';
const authToken = new URLSearchParams(window.location.search).get('token') || '';
function q(id) { return encodeURIComponent(document.getElementById(id).value); }
function apiPath(path) {
  if (!authToken) return path;
  return path + (path.includes('?') ? '&' : '?') + 'token=' + encodeURIComponent(authToken);
}
function bridgeQuery() { return 'key=' + q('bridgeKey') + '&key_type=' + q('bridgeKeyType'); }
function codegenRootQuery() {
  const root = document.getElementById('codegenRoot').value.trim();
  return root ? '&root=' + encodeURIComponent(root) : '';
}
function codegenQuery() { return 'config=' + q('codegenConfig') + codegenRootQuery(); }
function profileFileQuery() { return 'profile_file=' + q('profileFile') + codegenRootQuery(); }
async function callApi(path, options = {}) {
  const method = options.method || 'GET';
  out.textContent = method + ' ' + path + '\n';
  detail.className = 'detail';
  detail.textContent = '';
  const response = await fetch(apiPath(path), options);
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
  } else if (data.view === 'status' && (data.remaining || Array.isArray(data.comparisons))) {
    renderComparisonStatus(data);
  } else if (Array.isArray(data.profiles) && typeof data.profileCount === 'number') {
    renderProfileCheck(data);
  } else if (Array.isArray(data.steps)) {
    detail.textContent = renderSetupAssistant(data.steps);
  } else if (data.explain && typeof data.explain === 'object') {
    detail.textContent = renderCodegenExplain(data.explain);
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
function renderComparisonStatus(data) {
  detail.className = 'detail profile-check';
  const entries = Array.isArray(data.comparisons) ? data.comparisons : [data];
  const cards = entries.map(entry => {
    const parity = entry.parity || {};
    const remaining = entry.remaining || {};
    const nextBatch = entry.nextBatch || {};
    const topGap = entry.topGap || {};
    const commands = entry.commands || {};
    const project = parity.project || topGap.project || 'gemstone-rs';
    return '<div class="compare-card">' +
      '<h3>' + escapeHtml(entry.comparison || 'comparison') + '</h3>' +
      '<p>' + escapeHtml(entry.answer || '-') + '</p>' +
      '<div class="compare-score">' +
        '<span>gemstone-py ' + Number(parity.gemstonePyScore || 0) + '/' + Number(parity.maxScore || 0) + '</span>' +
        '<span>' + escapeHtml(project) + ' ' + Number(parity.projectScore || 0) + '/' + Number(parity.maxScore || 0) + '</span>' +
        '<span>gap ' + Number(parity.scoreGap || 0) + '</span>' +
      '</div>' +
      '<p><strong>Remaining:</strong> ' + Number(remaining.totalBatches || 0) + ' batches, ' + Number(remaining.hoursMin || 0) + '-' + Number(remaining.hoursMax || 0) + ' hours</p>' +
      '<p><strong>Next batch:</strong> ' + escapeHtml(nextBatch.focus || '-') + ' (' + Number(nextBatch.hoursMin || 0) + '-' + Number(nextBatch.hoursMax || 0) + ' hours)</p>' +
      '<p><strong>Top gap:</strong> ' + escapeHtml((topGap.priority || '-') + ' ' + (topGap.area || '-')) + '</p>' +
      '<p><strong>Next action:</strong> ' + escapeHtml(topGap.nextAction || '-') + '</p>' +
      '<p><strong>Commands:</strong><br>' + escapeHtml([commands.status, commands.scorecard, commands.parity, commands.batches].filter(Boolean).join('\n')) + '</p>' +
    '</div>';
  }).join('');
  const total = data.comparison === 'all'
    ? '<p><strong>Total:</strong> ' + Number(data.totalBatches || 0) + ' batches, ' + Number(data.hoursMin || 0) + '-' + Number(data.hoursMax || 0) + ' hours</p>'
    : '';
  detail.innerHTML = '<div class="pane-title">Comparison Status</div>' + total + cards;
}
function renderSetupAssistant(steps) {
  return steps.map(step => {
    const status = step.ok ? 'OK' : 'Needs attention';
    const action = step.action ? '\n  Action: ' + step.action : '';
    return status + ': ' + step.name + '\n  ' + (step.detail || '-') + action;
  }).join('\n\n');
}
function renderProfileCheck(data) {
  detail.className = 'detail profile-check';
  const profiles = data.profiles || [];
  const rows = profiles.map(profile => {
    const status = profile.ok ? 'ok' : (profile.error ? 'error' : 'stale');
    const statusClass = profile.ok ? 'profile-ok' : (profile.error ? 'profile-error' : 'profile-stale');
    return '<tr>' +
      '<td><strong>' + escapeHtml(profile.name || '(unnamed)') + '</strong></td>' +
      '<td class="' + statusClass + '">' + status + '</td>' +
      '<td>' + escapeHtml(profile.config || '-') + '</td>' +
      '<td>' + escapeHtml(profile.output || '-') + '</td>' +
      '<td>' + escapeHtml('exists=' + profile.exists + ' upToDate=' + profile.upToDate) + (profile.error ? '<br><span class="profile-error">' + escapeHtml(profile.error) + '</span>' : '') + '</td>' +
      '<td>' +
        '<button class="secondary" data-profile-action="preview" data-profile="' + escapeHtml(profile.name || '') + '">Preview</button>' +
        '<button class="secondary" data-profile-action="diff" data-profile="' + escapeHtml(profile.name || '') + '">Diff</button>' +
        '<button class="secondary" data-profile-action="check" data-profile="' + escapeHtml(profile.name || '') + '">Check</button>' +
        '<button class="secondary" data-profile-action="generate" data-profile="' + escapeHtml(profile.name || '') + '">Generate</button>' +
      '</td>' +
    '</tr>';
  }).join('');
  detail.innerHTML =
    '<div class="pane-title">Project Profile Check</div>' +
    '<p>Project profiles: ' + escapeHtml(data.profileFile || data.path || '-') + '</p>' +
    '<p>Summary: ' + Number(data.okCount || 0) + ' ok, ' + Number(data.staleCount || 0) + ' stale, ' + Number(data.errorCount || 0) + ' errors, ' + Number(data.profileCount || profiles.length) + ' total</p>' +
    '<table class="profile-table"><thead><tr><th>Profile</th><th>Status</th><th>Config</th><th>Output</th><th>Freshness</th><th>Actions</th></tr></thead><tbody>' + rows + '</tbody></table>';
  detail.querySelectorAll('button[data-profile-action]').forEach(button => {
    button.addEventListener('click', () => runProfileAction(button.dataset.profile, button.dataset.profileAction));
  });
}
function runProfileAction(profile, action) {
  if (profile) document.getElementById('profileName').value = profile;
  if (action === 'preview') codegenPreviewProfile();
  else if (action === 'diff') codegenDiffProfile();
  else if (action === 'check') codegenCheckProfile();
  else if (action === 'generate') codegenGenerateProfile();
}
function renderProfileCheckText(data) {
  const lines = [
    'Project profiles: ' + (data.profileFile || data.path || '-'),
    'Summary: ' + data.okCount + ' ok, ' + data.staleCount + ' stale, ' + data.errorCount + ' errors, ' + data.profileCount + ' total',
    ''
  ];
  for (const profile of data.profiles || []) {
    const status = profile.ok ? 'OK' : (profile.error ? 'ERROR' : 'STALE');
    lines.push(status + ': ' + profile.name);
    lines.push('  config: ' + (profile.config || '-'));
    lines.push('  output: ' + (profile.output || '-'));
    lines.push('  exists: ' + profile.exists + ' upToDate: ' + profile.upToDate);
    if (profile.error) lines.push('  error: ' + profile.error);
  }
  return lines.join('\n');
}
function renderCodegenExplain(explain) {
  const lines = [
    'Output: ' + (explain.output || '-'),
    'Test stubs: ' + ((explain.testStubs || []).join(', ') || '-'),
    '',
    'Classes:'
  ];
    for (const cls of explain.classes || []) {
    lines.push('  ' + cls.name + ' methods=' + ((cls.methods || []).length));
    for (const method of cls.methods || []) {
      const args = (method.arguments || []).length
        ? method.arguments.map(arg => arg.name + ':' + arg.type + ' as ' + arg.rustType).join(', ')
        : (method.args || []).join(', ');
      lines.push('    ' + method.selector + '(' + args + ') -> ' + method.return);
    }
  }
  lines.push('', 'Mapped:');
  for (const mapped of explain.mapped || []) {
    lines.push('  ' + mapped.name + ' fields=' + ((mapped.fields || []).length));
    for (const field of mapped.fields || []) {
      lines.push('    ' + field.name + ' key=' + field.key + ' keyType=' + field.keyType + ' type=' + field.type);
    }
  }
  return lines.join('\n');
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
  const ids = ['dictionary', 'className', 'protocol', 'selector', 'envFilePath', 'bridgeKey', 'bridgeValue', 'bridgeKeyType', 'bridgeValueType', 'codegenConfig', 'codegenRoot', 'configEditor', 'mappedName', 'mappingClass', 'profileName', 'profileFile'];
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
  loadRecentCodegenConfigs();
  loadCodegenProfiles();
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
  const response = await fetch(apiPath(path));
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
  const response = await fetch(apiPath('/api/bridge/keys'));
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
  if (!confirmWrite('Put this value into BridgeRoot?')) return;
  callApi('/api/bridge/put?' + bridgeQuery() + '&value=' + q('bridgeValue') + '&value_type=' + q('bridgeValueType'));
}
function removeBridgeValue() {
  if (!confirmWrite('Remove this key from BridgeRoot?')) return;
  callApi('/api/bridge/remove?' + bridgeQuery());
}
function compareGemstonePy() {
  callApi('/api/compare/gemstone-py/status');
}
function compareAll() {
  callApi('/api/compare/all/status');
}
async function showEnvTemplate() {
  await callApi('/api/env/sample');
}
async function writeEnvTemplate() {
  if (!confirmWrite('Write the environment template file?')) return;
  const query = 'path=' + q('envFilePath') + codegenRootQuery();
  await callApi('/api/env/write?' + query, { method: 'POST' });
}
function setupAssistantQuery() {
  return 'env_file=' + q('envFilePath') + '&' + codegenQuery() + '&profile_file=' + q('profileFile');
}
function profileCodegenQuery() {
  return 'profile=' + q('profileName') + '&' + profileFileQuery();
}
function runSetupAssistant() {
  rememberCodegenConfig();
  callApi('/api/setup/assistant?' + setupAssistantQuery());
}
function setConfigEditor(source) {
  const editor = document.getElementById('configEditor');
  editor.value = source;
  try { localStorage.setItem('gemstone-rs-explorer:configEditor', source); } catch {}
}
function setCodegenConfig(value) {
  const input = document.getElementById('codegenConfig');
  input.value = value;
  try { localStorage.setItem('gemstone-rs-explorer:codegenConfig', value); } catch {}
  rememberCodegenConfig(value);
}
function loadRecentCodegenConfigs() {
  let configs = [];
  try {
    const raw = localStorage.getItem(recentConfigsKey);
    configs = raw ? JSON.parse(raw) : [];
  } catch {
    configs = [];
  }
  if (!Array.isArray(configs)) configs = [];
  renderRecentCodegenConfigs(configs.filter(value => typeof value === 'string' && value.trim()));
}
function renderRecentCodegenConfigs(configs) {
  const picker = document.getElementById('codegenRecent');
  picker.innerHTML = '';
  const empty = document.createElement('option');
  empty.value = '';
  empty.textContent = configs.length ? 'Select a recent config' : 'No recent configs';
  picker.appendChild(empty);
  for (const config of configs) {
    const option = document.createElement('option');
    option.value = config;
    option.textContent = config;
    picker.appendChild(option);
  }
}
function rememberCodegenConfig(value) {
  const config = String(value || document.getElementById('codegenConfig').value || '').trim();
  if (!config) return;
  let configs = [];
  try {
    const raw = localStorage.getItem(recentConfigsKey);
    configs = raw ? JSON.parse(raw) : [];
  } catch {
    configs = [];
  }
  if (!Array.isArray(configs)) configs = [];
  configs = [config, ...configs.filter(item => item !== config)].slice(0, 8);
  try { localStorage.setItem(recentConfigsKey, JSON.stringify(configs)); } catch {}
  renderRecentCodegenConfigs(configs);
}
function readCodegenProfiles() {
  let profiles = [];
  try {
    const raw = localStorage.getItem(profilesKey);
    profiles = raw ? JSON.parse(raw) : [];
  } catch {
    profiles = [];
  }
  if (!Array.isArray(profiles)) return [];
  return profiles.map(normalizeCodegenProfile).filter(Boolean);
}
function normalizeCodegenProfile(profile) {
  if (!profile || typeof profile !== 'object') return null;
  const name = String(profile.name || '').trim();
  if (!name) return null;
  return {
    name,
    config: String(profile.config || '').trim(),
    root: String(profile.root || '').trim(),
    mapped: String(profile.mapped || '').trim(),
    className: String(profile.className || profile.class || '').trim()
  };
}
function writeCodegenProfiles(profiles) {
  const clean = profiles.map(normalizeCodegenProfile).filter(Boolean);
  try { localStorage.setItem(profilesKey, JSON.stringify(clean)); } catch {}
  renderCodegenProfiles(clean);
}
function profileFingerprint(profile) {
  const clean = normalizeCodegenProfile(profile);
  return clean ? JSON.stringify(clean) : '';
}
function renderImportSummary(summary) {
  importSummary.textContent = [
    'New: ' + (summary.created.length ? summary.created.join(', ') : '-'),
    'Replaced: ' + (summary.replaced.length ? summary.replaced.join(', ') : '-'),
    'Unchanged: ' + (summary.unchanged.length ? summary.unchanged.join(', ') : '-')
  ].join('\n');
}
function loadCodegenProfiles() {
  renderCodegenProfiles(readCodegenProfiles());
}
function renderCodegenProfiles(profiles) {
  const picker = document.getElementById('codegenProfile');
  picker.innerHTML = '';
  const empty = document.createElement('option');
  empty.value = '';
  empty.textContent = profiles.length ? 'Select a saved profile' : 'No saved profiles';
  picker.appendChild(empty);
  for (const profile of profiles) {
    const option = document.createElement('option');
    option.value = profile.name;
    option.textContent = profile.name;
    if (profile.name === document.getElementById('profileName').value.trim()) option.selected = true;
    picker.appendChild(option);
  }
}
function currentCodegenProfile() {
  const name = document.getElementById('profileName').value.trim()
    || document.getElementById('codegenConfig').value.trim()
    || 'default';
  return {
    name,
    config: document.getElementById('codegenConfig').value.trim(),
    root: document.getElementById('codegenRoot').value.trim(),
    mapped: document.getElementById('mappedName').value.trim(),
    className: document.getElementById('mappingClass').value.trim()
  };
}
function profileExportPayload(profile) {
  return {
    kind: 'gemstone-rs-explorer-codegen-profile',
    version: 1,
    profile
  };
}
function profilesExportPayload(profiles) {
  return {
    kind: 'gemstone-rs-explorer-codegen-profiles',
    version: 1,
    profiles
  };
}
function applyCodegenProfile(profile) {
  document.getElementById('profileName').value = profile.name || 'default';
  if (profile.config) document.getElementById('codegenConfig').value = profile.config;
  document.getElementById('codegenRoot').value = profile.root || '';
  document.getElementById('mappedName').value = profile.mapped || 'BookingDraft';
  document.getElementById('mappingClass').value = profile.className || 'Object';
  for (const id of ['profileName', 'codegenConfig', 'codegenRoot', 'mappedName', 'mappingClass']) {
    try { localStorage.setItem('gemstone-rs-explorer:' + id, document.getElementById(id).value); } catch {}
  }
  rememberCodegenConfig(profile.config);
  loadCodegenConfigs();
  loadCodegenConfig();
}
function saveCodegenProfile() {
  const profile = currentCodegenProfile();
  const profiles = [profile, ...readCodegenProfiles().filter(item => item.name !== profile.name)].slice(0, 16);
  writeCodegenProfiles(profiles);
  detail.textContent = 'Saved codegen profile: ' + profile.name;
}
function exportCodegenProfile() {
  const profile = currentCodegenProfile();
  saveCodegenProfile();
  const source = JSON.stringify(profileExportPayload(profile), null, 2);
  document.getElementById('profileJson').value = source;
  detail.textContent = source;
}
function importCodegenProfile() {
  const source = document.getElementById('profileJson').value;
  importCodegenProfilesSource(source, true);
}
function importCodegenProfilesSource(source, applyFirst) {
  let payload;
  try {
    payload = JSON.parse(source);
  } catch (error) {
    detail.textContent = 'Profile import failed: ' + error.message;
    return;
  }
  const candidates = Array.isArray(payload)
    ? payload
    : Array.isArray(payload.profiles)
      ? payload.profiles
      : [payload.profile || payload];
  const imported = candidates.map(normalizeCodegenProfile).filter(Boolean);
  if (!imported.length) {
    detail.textContent = 'Profile import failed: no valid profiles found.';
    return;
  }
  const current = readCodegenProfiles();
  const created = [];
  const replaced = [];
  const unchanged = [];
  for (const profile of imported) {
    const existing = current.find(item => item.name === profile.name);
    if (!existing) created.push(profile.name);
    else if (profileFingerprint(existing) === profileFingerprint(profile)) unchanged.push(profile.name);
    else replaced.push(profile.name);
  }
  const existing = current.filter(item => !imported.some(profile => profile.name === item.name));
  const merged = [...imported, ...existing].slice(0, 16);
  writeCodegenProfiles(merged);
  if (applyFirst) applyCodegenProfile(imported[0]);
  const message = imported.length === 1
    ? 'Imported codegen profile: ' + imported[0].name
    : 'Imported ' + imported.length + ' codegen profiles.';
  renderImportSummary({ created, replaced, unchanged });
  detail.textContent = [
    message,
    'New: ' + (created.length ? created.join(', ') : '-'),
    'Replaced: ' + (replaced.length ? replaced.join(', ') : '-'),
    'Unchanged: ' + (unchanged.length ? unchanged.join(', ') : '-')
  ].join('\n');
}
function loadCodegenProfile() {
  const name = document.getElementById('codegenProfile').value;
  if (!name) return;
  const profile = readCodegenProfiles().find(item => item.name === name);
  if (profile) applyCodegenProfile(profile);
}
function deleteCodegenProfile() {
  const name = document.getElementById('profileName').value.trim() || document.getElementById('codegenProfile').value;
  if (!name) return;
  const profiles = readCodegenProfiles().filter(item => item.name !== name);
  writeCodegenProfiles(profiles);
  detail.textContent = 'Deleted codegen profile: ' + name;
}
async function loadProjectProfiles() {
  const data = await callApi('/api/codegen/profiles?' + profileFileQuery());
  if (typeof data.source === 'string') {
    document.getElementById('profileJson').value = data.source;
    importCodegenProfilesSource(data.source, false);
  }
}
function checkProjectProfiles() {
  callApi('/api/codegen/profiles/check?' + profileFileQuery());
}
async function saveProjectProfiles() {
  if (!confirmWrite('Write project profiles to disk?')) return;
  const profiles = readCodegenProfiles();
  const source = JSON.stringify(profilesExportPayload(profiles), null, 2);
  document.getElementById('profileJson').value = source;
  const data = await callApi('/api/codegen/profiles/save?' + profileFileQuery(), {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: source
  });
  if (data.success) detail.textContent = 'Saved project profiles: ' + data.profileFile;
}
async function loadCodegenConfig() {
  const data = await callApi('/api/codegen/config?' + codegenQuery());
  if (typeof data.source === 'string') {
    setConfigEditor(data.source);
    rememberCodegenConfig();
  }
}
async function loadCodegenConfigs() {
  const root = document.getElementById('codegenRoot').value.trim();
  const data = await callApi('/api/codegen/configs' + (root ? '?root=' + encodeURIComponent(root) : ''));
  const picker = document.getElementById('codegenConfigPicker');
  picker.innerHTML = '';
  const empty = document.createElement('option');
  empty.value = '';
  empty.textContent = data.configs && data.configs.length ? 'Select a config' : 'No .codegen files found';
  picker.appendChild(empty);
  for (const config of data.configs || []) {
    const option = document.createElement('option');
    option.value = config;
    option.textContent = config;
    if (config === document.getElementById('codegenConfig').value) option.selected = true;
    picker.appendChild(option);
  }
}
function pickCodegenConfig() {
  const picker = document.getElementById('codegenConfigPicker');
  if (!picker.value) return;
  setCodegenConfig(picker.value);
  loadCodegenConfig();
}
function pickRecentCodegenConfig() {
  const picker = document.getElementById('codegenRecent');
  if (!picker.value) return;
  setCodegenConfig(picker.value);
  loadCodegenConfig();
}
async function saveCodegenConfig() {
  if (!confirmWrite('Write this codegen config to disk?')) return;
  const source = document.getElementById('configEditor').value;
  const data = await callApi('/api/codegen/config/save?' + codegenQuery(), {
    method: 'POST',
    headers: { 'Content-Type': 'text/plain; charset=utf-8' },
    body: source
  });
  if (data.success) rememberCodegenConfig();
}
async function codegenSample() {
  const data = await callApi('/api/codegen/sample');
  if (typeof data.config === 'string') setConfigEditor(data.config);
}
async function discoverMappingConfig() {
  const data = await callApi('/api/codegen/discover-mapping?' + codegenQuery() + '&mapped=' + q('mappedName') + '&class=' + q('mappingClass'));
  if (typeof data.config === 'string') setConfigEditor(data.config);
}
function codegenPreview() { rememberCodegenConfig(); callApi('/api/codegen/preview?' + codegenQuery()); }
function codegenPreviewProfile() { rememberCodegenConfig(); callApi('/api/codegen/preview-profile?' + profileCodegenQuery()); }
function codegenExplain() { rememberCodegenConfig(); callApi('/api/codegen/explain?' + codegenQuery()); }
function codegenExplainProfile() { rememberCodegenConfig(); callApi('/api/codegen/explain-profile?' + profileCodegenQuery()); }
function codegenDiff() { rememberCodegenConfig(); callApi('/api/codegen/diff?' + codegenQuery()); }
function codegenDiffProfile() { rememberCodegenConfig(); callApi('/api/codegen/diff-profile?' + profileCodegenQuery()); }
function codegenCheck() { rememberCodegenConfig(); callApi('/api/codegen/check?' + codegenQuery()); }
function codegenCheckProfile() { rememberCodegenConfig(); callApi('/api/codegen/check-profile?' + profileCodegenQuery()); }
function codegenGenerate() {
  if (!confirmWrite('Generate wrappers and overwrite the configured output file?')) return;
  rememberCodegenConfig();
  callApi('/api/codegen/generate?' + codegenQuery());
}
function codegenGenerateProfile() {
  if (!confirmWrite('Generate wrappers for this profile and overwrite its output file?')) return;
  rememberCodegenConfig();
  callApi('/api/codegen/generate-profile?' + profileCodegenQuery());
}
function confirmWrite(message) {
  return window.confirm(message + '\n\nThis only works when the explorer was started with --allow-write.');
}
persistFields();
loadRecentCodegenConfigs();
loadCodegenProfiles();
loadCodegenConfigs();
</script>
</body>
</html>"#,
    );
    html
}

fn config_json(config: &ExplorerConfig) -> String {
    format!(
        r#"{{"host":"{}","port":{},"readOnly":{},"allowEval":{},"codegenRoot":"{}","allowAbsoluteWritePaths":{},"envFile":{},"loopbackOnly":true,"authRequired":{}}}"#,
        config.host,
        config.port,
        config.read_only,
        config.allow_eval,
        escape_json(&config.codegen_root.display().to_string()),
        config.allow_absolute_write_paths,
        optional_json_string(
            config
                .env_file
                .as_deref()
                .map(|path| path.display().to_string())
                .as_deref()
        ),
        config.auth_token.is_some()
    )
}

fn request_is_authorized(request: &HttpRequest, route: &Route, config: &ExplorerConfig) -> bool {
    let Some(expected) = config.auth_token.as_deref() else {
        return true;
    };
    if route.path == "/health" {
        return true;
    }

    route
        .query("token")
        .as_deref()
        .is_some_and(|token| token_eq(token, expected))
        || request
            .header("x-gemstone-rs-token")
            .is_some_and(|token| token_eq(token, expected))
        || request
            .header("authorization")
            .and_then(|value| value.strip_prefix("Bearer "))
            .is_some_and(|token| token_eq(token, expected))
}

fn token_eq(candidate: &str, expected: &str) -> bool {
    let candidate = candidate.as_bytes();
    let expected = expected.as_bytes();
    if candidate.len() != expected.len() {
        return false;
    }
    let mut diff = 0_u8;
    for (left, right) in candidate.iter().zip(expected.iter()) {
        diff |= left ^ right;
    }
    diff == 0
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
        401 => "Unauthorized",
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
    "usage: gemstone-rs-explorer [--env-file .env.gemstone-rs] [--auth-token TOKEN|--auth-token-env NAME] [--host 127.0.0.1] [--port 8787] [--codegen-root .] [--allow-eval] [--allow-write] [--allow-absolute-write-paths]"
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
        assert_eq!(config.codegen_root, PathBuf::from("."));
        assert_eq!(config.env_file, None);
        assert_eq!(config.auth_token, None);
    }

    #[test]
    fn config_rejects_non_loopback_host() {
        let err = ExplorerConfig::parse(&args(&["--host", "0.0.0.0"])).unwrap_err();
        assert!(err.to_string().contains("loopback"));
    }

    #[test]
    fn config_accepts_eval_flag_and_port() {
        let config = ExplorerConfig::parse(&args(&[
            "--port",
            "9000",
            "--allow-eval",
            "--env-file=.env.gemstone-rs",
            "--codegen-root",
            "examples",
        ]))
        .unwrap();
        assert_eq!(config.port, 9000);
        assert!(config.allow_eval);
        assert_eq!(config.codegen_root, PathBuf::from("examples"));
        assert_eq!(config.env_file, Some(PathBuf::from(".env.gemstone-rs")));
    }

    #[test]
    fn config_accepts_auth_token() {
        let config = ExplorerConfig::parse(&args(&["--auth-token", "secret"])).unwrap();
        assert_eq!(config.auth_token.as_deref(), Some("secret"));

        let err = ExplorerConfig::parse(&args(&["--auth-token="])).unwrap_err();
        assert!(err.to_string().contains("must not be empty"));
    }

    #[test]
    fn env_file_parser_reads_safe_template_values() {
        let values = parse_env_file_source(
            "export GS_USERNAME='Data'\"'\"'Curator'\nGS_PASSWORD='<set-your-password>'\nGEMSTONE=/opt/gemstone # product root\n",
            Path::new(".env.gemstone-rs"),
        )
        .unwrap();
        assert_eq!(
            values,
            vec![
                ("GS_USERNAME".to_string(), "Data'Curator".to_string()),
                ("GEMSTONE".to_string(), "/opt/gemstone".to_string())
            ]
        );
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
        assert!(response.body.contains(r#""codegenRoot":".""#));
        assert!(response.body.contains(r#""envFile":null"#));
        assert!(response.body.contains(r#""authRequired":false"#));
    }

    #[test]
    fn auth_token_protects_explorer_routes_when_configured() {
        let config = ExplorerConfig {
            auth_token: Some("secret".to_string()),
            ..ExplorerConfig::default()
        };

        let missing = handle_request("GET /api/config HTTP/1.1", &config);
        assert_eq!(missing.status, 401);

        let query = handle_request("GET /api/config?token=secret HTTP/1.1", &config);
        assert_eq!(query.status, 200);

        let request = HttpRequest {
            method: "GET".to_string(),
            target: "/api/config".to_string(),
            headers: vec![("X-GemStone-RS-Token".to_string(), "secret".to_string())],
            body: String::new(),
        };
        let header = handle_http_request(&request, &config);
        assert_eq!(header.status, 200);
    }

    #[test]
    fn doctor_endpoint_reports_environment() {
        let response = handle_request("GET /api/doctor HTTP/1.1", &ExplorerConfig::default());
        assert!(matches!(response.status, 200 | 503), "{}", response.body);
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
        assert!(response.body.contains("codegenRoot"));
        assert!(response.body.contains("codegenRootQuery()"));
        assert!(response.body.contains("codegenConfigPicker"));
        assert!(response.body.contains("codegenRecent"));
        assert!(response.body.contains("profileName"));
        assert!(response.body.contains("codegenProfile"));
        assert!(response.body.contains("profileJson"));
        assert!(response.body.contains("profileFile"));
        assert!(response.body.contains("importSummary"));
        assert!(response.body.contains("Comparison Status"));
        assert!(response.body.contains("compareGemstonePy()"));
        assert!(response.body.contains("compareAll()"));
        assert!(response.body.contains("renderComparisonStatus"));
        assert!(response.body.contains("configEditor"));
        assert!(response.body.contains("envFilePath"));
        assert!(response.body.contains("showEnvTemplate()"));
        assert!(response.body.contains("writeEnvTemplate()"));
        assert!(response.body.contains("loadCodegenConfigs()"));
        assert!(response.body.contains("pickCodegenConfig()"));
        assert!(response.body.contains("loadRecentCodegenConfigs()"));
        assert!(response.body.contains("pickRecentCodegenConfig()"));
        assert!(response.body.contains("loadCodegenProfiles()"));
        assert!(response.body.contains("saveCodegenProfile()"));
        assert!(response.body.contains("exportCodegenProfile()"));
        assert!(response.body.contains("importCodegenProfile()"));
        assert!(response.body.contains("renderImportSummary"));
        assert!(response.body.contains("loadProjectProfiles()"));
        assert!(response.body.contains("checkProjectProfiles()"));
        assert!(response.body.contains("saveProjectProfiles()"));
        assert!(response.body.contains("deleteCodegenProfile()"));
        assert!(response
            .body
            .contains("gemstone-rs-explorer-codegen-profile"));
        assert!(response.body.contains("/api/codegen/profiles"));
        assert!(response.body.contains("rememberCodegenConfig"));
        assert!(response.body.contains("loadCodegenConfig()"));
        assert!(response.body.contains("saveCodegenConfig()"));
        assert!(response.body.contains("method: 'POST'"));
        assert!(response.body.contains("discoverMappingConfig()"));
        assert!(response.body.contains("codegenExplain()"));
        assert!(response.body.contains("codegenExplainProfile()"));
        assert!(response.body.contains("codegenPreviewProfile()"));
        assert!(response.body.contains("codegenDiffProfile()"));
        assert!(response.body.contains("codegenCheckProfile()"));
        assert!(response.body.contains("codegenGenerateProfile()"));
        assert!(response.body.contains("bridgeKeyType"));
        assert!(response.body.contains("bridgeValueType"));
        assert!(response.body.contains("Generated Source / Config / Diff"));
        assert!(response.body.contains("renderDiff"));
        assert!(response.body.contains("renderSideBySideDiff"));
        assert!(response.body.contains("Side-by-Side Diff"));
        assert!(response.body.contains("localStorage"));
        assert!(response.body.contains("read_only=true"));
        assert!(response.body.contains("allow_eval=false"));
        assert!(response.body.contains("auth_required=false"));
        assert!(response.body.contains("apiPath(path)"));
        assert!(response.body.contains("confirmWrite"));
        assert!(response.body.contains("/api/doctor"));
        assert!(response.body.contains("/api/setup/assistant"));
        assert!(response.body.contains("/api/env/sample"));
        assert!(response.body.contains("/api/env/write"));
        assert!(response.body.contains("/api/compare/gemstone-py/status"));
        assert!(response.body.contains("/api/compare/all/status"));
        assert!(response.body.contains("/api/bridge/keys"));
        assert!(response.body.contains("/api/bridge/put"));
        assert!(response.body.contains("/api/codegen/configs"));
        assert!(response.body.contains("/api/codegen/profiles/check"));
        assert!(response.body.contains("/api/codegen/preview-profile"));
        assert!(response.body.contains("/api/codegen/explain"));
        assert!(response.body.contains("/api/codegen/explain-profile"));
        assert!(response.body.contains("/api/codegen/diff-profile"));
        assert!(response.body.contains("/api/codegen/check-profile"));
        assert!(response.body.contains("/api/codegen/generate-profile"));
        assert!(response.body.contains("/api/codegen/check"));
    }

    #[test]
    fn compare_status_endpoint_reports_remaining_batches() {
        let response = handle_request(
            "GET /api/compare/gemstone-py/status HTTP/1.1",
            &ExplorerConfig::default(),
        );
        assert_eq!(response.status, 200);
        assert!(response.body.contains(r#""comparison":"gemstone-py""#));
        assert!(response.body.contains(r#""view":"status""#));
        assert!(response.body.contains(r#""totalBatches":6"#));
        assert!(response.body.contains(r#""hoursMin":40"#));
        assert!(response.body.contains(r#""hoursMax":71"#));
        assert!(response.body.contains(r#""project":"gemstone-rs""#));
        assert!(response.body.contains("Explorer and VS Code visual polish"));
        assert!(response
            .body
            .contains("gemstone-rs compare gemstone-py --batches"));
    }

    #[test]
    fn compare_all_status_endpoint_reports_combined_batches() {
        let response = handle_request(
            "GET /api/compare/all/status HTTP/1.1",
            &ExplorerConfig::default(),
        );
        assert_eq!(response.status, 200);
        assert!(response.body.contains(r#""comparison":"all""#));
        assert!(response.body.contains(r#""totalBatches":12"#));
        assert!(response.body.contains(r#""hoursMin":82"#));
        assert!(response.body.contains(r#""hoursMax":143"#));
        assert!(response.body.contains(r#""comparison":"gemstone-py""#));
        assert!(response.body.contains(r#""comparison":"gemstone-js""#));
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
        assert_eq!(
            bridge_value_type_query(Some("Symbol")),
            BridgeValueType::Symbol
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
    fn env_sample_endpoint_returns_safe_template() {
        let response = handle_request("GET /api/env/sample HTTP/1.1", &ExplorerConfig::default());
        assert_eq!(response.status, 200);
        assert!(response.body.contains(r#""success":true"#));
        assert!(
            response
                .body
                .contains("export GS_PASSWORD='&lt;set-your-password&gt;'")
                || response
                    .body
                    .contains("export GS_PASSWORD='<set-your-password>'")
        );
        assert!(!response.body.contains("change-me"));
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
    fn codegen_explain_endpoint_reports_structured_summary() {
        let root =
            fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap();
        let config = ExplorerConfig {
            codegen_root: root,
            ..ExplorerConfig::default()
        };
        let response = handle_request(
            "GET /api/codegen/explain?config=examples/codegen/gemstone-rs.codegen HTTP/1.1",
            &config,
        );
        assert_eq!(response.status, 200);
        assert!(response.body.contains(r#""success":true"#));
        assert!(response.body.contains(r#""explain":"#));
        assert!(response
            .body
            .contains(r#""testStubs":["generated_surface_names_are_stable"]"#));
        assert!(response.body.contains(r#""selector":"printString""#));
    }

    #[test]
    fn codegen_explain_profile_endpoint_reports_structured_summary() {
        let root =
            fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap();
        let config = ExplorerConfig {
            codegen_root: root,
            ..ExplorerConfig::default()
        };
        let response = handle_request(
            "GET /api/codegen/explain-profile?profile=default&profile_file=examples/codegen/gemstone-rs.codegen-profiles.json HTTP/1.1",
            &config,
        );
        assert_eq!(response.status, 200);
        assert!(response.body.contains(r#""success":true"#));
        assert!(response.body.contains(r#""profile":"default""#));
        assert!(response.body.contains(r#""explain":"#));
        assert!(response
            .body
            .contains(r#""testStubs":["generated_surface_names_are_stable"]"#));
        assert!(response.body.contains(r#""selector":"printString""#));
    }

    #[test]
    fn codegen_profile_preview_diff_and_check_use_project_profile() {
        let root =
            fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap();
        let config = ExplorerConfig {
            codegen_root: root,
            ..ExplorerConfig::default()
        };

        let preview = handle_request(
            "GET /api/codegen/preview-profile?profile=default&profile_file=examples/codegen/gemstone-rs.codegen-profiles.json HTTP/1.1",
            &config,
        );
        assert_eq!(preview.status, 200);
        assert!(preview.body.contains(r#""profile":"default""#));
        assert!(preview.body.contains(r#""source":"#));
        assert!(preview.body.contains("pub struct Object"));

        let check = handle_request(
            "GET /api/codegen/check-profile?profile=default&profile_file=examples/codegen/gemstone-rs.codegen-profiles.json HTTP/1.1",
            &config,
        );
        assert_eq!(check.status, 200);
        assert!(check.body.contains(r#""profile":"default""#));
        assert!(check.body.contains(r#""upToDate":true"#));

        let diff = handle_request(
            "GET /api/codegen/diff-profile?profile=default&profile_file=examples/codegen/gemstone-rs.codegen-profiles.json HTTP/1.1",
            &config,
        );
        assert_eq!(diff.status, 200);
        assert!(diff.body.contains(r#""profile":"default""#));
        assert!(diff.body.contains(r#""upToDate":true"#));
    }

    #[test]
    fn codegen_profiles_check_reports_all_project_profiles() {
        let root =
            fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap();
        let config = ExplorerConfig {
            codegen_root: root,
            ..ExplorerConfig::default()
        };

        let response = handle_request(
            "GET /api/codegen/profiles/check?profile_file=examples/codegen/gemstone-rs.codegen-profiles.json HTTP/1.1",
            &config,
        );
        assert_eq!(response.status, 200);
        assert!(response.body.contains(r#""success":true"#));
        assert!(response.body.contains(r#""ok":true"#));
        assert!(response.body.contains(r#""profileCount":3"#));
        assert!(response.body.contains(r#""okCount":3"#));
        assert!(response.body.contains(r#""staleCount":0"#));
        assert!(response.body.contains(r#""errorCount":0"#));
        assert!(response.body.contains(r#""name":"default""#));
        assert!(response.body.contains(r#""name":"object-wrapper""#));
        assert!(response.body.contains(r#""name":"bridge-mapping""#));
        assert!(response.body.contains("gemstone_wrappers.rs"));
    }

    #[test]
    fn codegen_generate_profile_is_disabled_by_default() {
        let response = handle_request(
            "GET /api/codegen/generate-profile?profile=default&profile_file=examples/codegen/gemstone-rs.codegen-profiles.json HTTP/1.1",
            &ExplorerConfig::default(),
        );
        assert_eq!(response.status, 403);
        assert!(response.body.contains("allow-write"));
    }

    #[test]
    fn codegen_explain_profile_reports_missing_profile() {
        let root =
            fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap();
        let config = ExplorerConfig {
            codegen_root: root,
            ..ExplorerConfig::default()
        };
        let response = handle_request(
            "GET /api/codegen/explain-profile?profile=missing&profile_file=examples/codegen/gemstone-rs.codegen-profiles.json HTTP/1.1",
            &config,
        );
        assert_eq!(response.status, 404);
        assert!(response.body.contains("profile not found"));
    }

    #[test]
    fn setup_assistant_reports_environment_and_codegen_steps() {
        let root =
            fs::canonicalize(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")).unwrap();
        let config = ExplorerConfig {
            codegen_root: root,
            ..ExplorerConfig::default()
        };
        let response = handle_request(
            "GET /api/setup/assistant?config=examples/codegen/gemstone-rs.codegen&profile_file=examples/codegen/gemstone-rs.codegen-profiles.json HTTP/1.1",
            &config,
        );
        assert!(matches!(response.status, 200 | 503), "{}", response.body);
        assert!(response.body.contains(r#""steps":["#));
        assert!(response.body.contains("Environment file"));
        assert!(response.body.contains("Codegen config"));
        assert!(response.body.contains("Project profiles"));
    }

    #[test]
    fn codegen_config_endpoint_uses_configured_root_for_relative_paths() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let config = ExplorerConfig {
            codegen_root: root,
            ..ExplorerConfig::default()
        };
        let response = handle_request(
            "GET /api/codegen/config?config=examples/codegen/gemstone-rs.codegen HTTP/1.1",
            &config,
        );
        assert_eq!(response.status, 200);
        assert!(response.body.contains(r#""success":true"#));
        assert!(response.body.contains("generated/gemstone_wrappers.rs"));
    }

    #[test]
    fn codegen_configs_endpoint_lists_known_configs() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let config = ExplorerConfig {
            codegen_root: root,
            ..ExplorerConfig::default()
        };
        let response = handle_request("GET /api/codegen/configs HTTP/1.1", &config);
        assert_eq!(response.status, 200);
        assert!(response.body.contains(r#""success":true"#));
        assert!(response
            .body
            .contains("examples/codegen/gemstone-rs.codegen"));
    }

    #[test]
    fn codegen_configs_endpoint_rejects_missing_root() {
        let response = handle_request(
            "GET /api/codegen/configs?root=target/definitely-missing-codegen-root HTTP/1.1",
            &ExplorerConfig::default(),
        );
        assert_eq!(response.status, 400);
        assert!(response.body.contains("root is not a directory"));
    }

    #[test]
    fn codegen_profiles_save_is_disabled_by_default() {
        let response = handle_request(
            "GET /api/codegen/profiles/save?profile_file=tmp.json HTTP/1.1",
            &ExplorerConfig::default(),
        );
        assert_eq!(response.status, 403);
        assert!(response.body.contains("allow-write"));
    }

    #[test]
    fn env_write_is_disabled_by_default() {
        let response = handle_request(
            "POST /api/env/write?path=.env.gemstone-rs HTTP/1.1\r\ncontent-length: 0\r\n\r\n",
            &ExplorerConfig::default(),
        );
        assert_eq!(response.status, 403);
        assert!(response.body.contains("writes disabled"));
    }

    #[test]
    fn env_write_writes_template_when_enabled() {
        let root = env::temp_dir().join(format!("gemstone-rs-explorer-env-{}", std::process::id()));
        let path = root.join(".env.gemstone-rs");
        let _ = fs::remove_dir_all(&root);
        let config = ExplorerConfig {
            read_only: false,
            codegen_root: root.clone(),
            ..ExplorerConfig::default()
        };
        let request = HttpRequest {
            method: "POST".to_string(),
            target: "/api/env/write?path=.env.gemstone-rs".to_string(),
            headers: Vec::new(),
            body: String::new(),
        };

        let response = handle_http_request(&request, &config);
        assert_eq!(response.status, 200);
        assert!(response.body.contains(r#""success":true"#));
        assert!(fs::read_to_string(&path)
            .unwrap()
            .contains("export GS_PASSWORD='<set-your-password>'"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn post_codegen_profiles_save_writes_request_body() {
        let root = env::temp_dir().join(format!(
            "gemstone-rs-explorer-profiles-{}",
            std::process::id()
        ));
        let path = root.join("profiles.json");
        let _ = fs::remove_dir_all(&root);
        let config = ExplorerConfig {
            read_only: false,
            codegen_root: root.clone(),
            ..ExplorerConfig::default()
        };
        let source =
            r#"{"kind":"gemstone-rs-explorer-codegen-profiles","version":1,"profiles":[]}"#;
        let request = HttpRequest {
            method: "POST".to_string(),
            target: "/api/codegen/profiles/save?profile_file=profiles.json".to_string(),
            headers: Vec::new(),
            body: source.to_string(),
        };

        let response = handle_http_request(&request, &config);
        assert_eq!(response.status, 200);
        assert!(response.body.contains(r#""success":true"#));
        assert_eq!(fs::read_to_string(&path).unwrap(), source);

        let read = handle_request(
            "GET /api/codegen/profiles?profile_file=profiles.json HTTP/1.1",
            &config,
        );
        assert_eq!(read.status, 200);
        assert!(read.body.contains("gemstone-rs-explorer-codegen-profiles"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn codegen_config_save_rejects_path_traversal() {
        let config = ExplorerConfig {
            read_only: false,
            ..ExplorerConfig::default()
        };
        let response = handle_request(
            "GET /api/codegen/config/save?config=../escape.codegen&source=output%20%3D%20tmp.rs HTTP/1.1",
            &config,
        );
        assert_eq!(response.status, 400);
        assert!(response.body.contains("path traversal"));
    }

    #[test]
    fn codegen_config_save_rejects_encoded_path_traversal() {
        let config = ExplorerConfig {
            read_only: false,
            ..ExplorerConfig::default()
        };
        let response = handle_request(
            "GET /api/codegen/config/save?config=%2e%2e/escape.codegen&source=output%20%3D%20tmp.rs HTTP/1.1",
            &config,
        );
        assert_eq!(response.status, 400);
        assert!(response.body.contains("path traversal"));
    }

    #[test]
    fn codegen_config_save_rejects_root_path_traversal() {
        let config = ExplorerConfig {
            read_only: false,
            ..ExplorerConfig::default()
        };
        let response = handle_request(
            "GET /api/codegen/config/save?root=../escape&config=draft.codegen&source=output%20%3D%20tmp.rs HTTP/1.1",
            &config,
        );
        assert_eq!(response.status, 400);
        assert!(response.body.contains("path traversal"));
    }

    #[test]
    fn codegen_config_save_rejects_absolute_root_query_by_default() {
        let config = ExplorerConfig {
            read_only: false,
            ..ExplorerConfig::default()
        };
        let root = env::temp_dir().join("gemstone-rs-absolute-root");
        let response = handle_request(
            &format!(
                "GET /api/codegen/config/save?root={}&config=draft.codegen&source=output%20%3D%20tmp.rs HTTP/1.1",
                root.display()
            ),
            &config,
        );
        assert_eq!(response.status, 400);
        assert!(response.body.contains("absolute paths are disabled"));
    }

    #[test]
    fn codegen_config_save_allows_absolute_target_when_explicitly_enabled() {
        let root = env::temp_dir().join(format!(
            "gemstone-rs-explorer-absolute-save-{}",
            std::process::id()
        ));
        let path = root.join("absolute.codegen");
        let _ = fs::remove_dir_all(&root);
        let config = ExplorerConfig {
            read_only: false,
            allow_absolute_write_paths: true,
            ..ExplorerConfig::default()
        };
        let source = "output = generated/absolute.rs\n";
        let request = HttpRequest {
            method: "POST".to_string(),
            target: format!("/api/codegen/config/save?config={}", path.display()),
            headers: Vec::new(),
            body: source.to_string(),
        };

        let response = handle_http_request(&request, &config);
        assert_eq!(response.status, 200);
        assert!(response.body.contains(r#""success":true"#));
        assert_eq!(fs::read_to_string(&path).unwrap(), source);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn codegen_profile_save_rejects_absolute_path_by_default() {
        let config = ExplorerConfig {
            read_only: false,
            ..ExplorerConfig::default()
        };
        let path = env::temp_dir().join("gemstone-rs-absolute-profile.json");
        let request = HttpRequest {
            method: "POST".to_string(),
            target: format!("/api/codegen/profiles/save?profile_file={}", path.display()),
            headers: Vec::new(),
            body: r#"{"kind":"gemstone-rs-explorer-codegen-profiles","version":1,"profiles":[]}"#
                .to_string(),
        };
        let response = handle_http_request(&request, &config);
        assert_eq!(response.status, 400);
        assert!(response.body.contains("absolute paths are disabled"));
    }

    #[test]
    fn codegen_profiles_save_rejects_invalid_profile_source() {
        let root = env::temp_dir().join(format!(
            "gemstone-rs-explorer-invalid-profiles-{}.json",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let config = ExplorerConfig {
            read_only: false,
            codegen_root: root.clone(),
            ..ExplorerConfig::default()
        };
        let request = HttpRequest {
            method: "POST".to_string(),
            target: "/api/codegen/profiles/save?profile_file=invalid-profiles.json".to_string(),
            headers: Vec::new(),
            body: "not json".to_string(),
        };

        let response = handle_http_request(&request, &config);
        assert_eq!(response.status, 400);
        assert!(response.body.contains("profile JSON parse error"));
        assert!(!root.join("invalid-profiles.json").exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn profile_schema_requires_kind_version_and_profile_names() {
        assert!(profiles::validate_source(
            r#"{"kind":"gemstone-rs-explorer-codegen-profiles","version":1,"profiles":[{"name":"default","config":"examples/codegen/gemstone-rs.codegen","root":"","mapped":"BookingDraft","className":"Object"}]}"#
        )
        .is_ok());
        assert_eq!(
            profiles::validate_source(r#"{"kind":"bad","version":1,"profiles":[]}"#)
                .unwrap_err()
                .to_string(),
            "kind must be gemstone-rs-explorer-codegen-profiles"
        );
        assert_eq!(
            profiles::validate_source(
                r#"{"kind":"gemstone-rs-explorer-codegen-profiles","version":1,"profiles":[{}]}"#
            )
            .unwrap_err()
            .to_string(),
            "profiles[0].name is required"
        );
        assert_eq!(
            profiles::validate_source(
                r#"{"kind":"gemstone-rs-explorer-codegen-profiles","version":1,"extra":true,"profiles":[]}"#
            )
            .unwrap_err()
            .to_string(),
            "extra is not supported"
        );
        assert_eq!(
            profiles::validate_source(
                r#"{"kind":"gemstone-rs-explorer-codegen-profiles","version":1,"profiles":[{"name":"default","unsupported":"value"}]}"#
            )
            .unwrap_err()
            .to_string(),
            "profiles[0].unsupported is not supported"
        );
        assert_eq!(
            profiles::validate_source(
                r#"{"kind":"gemstone-rs-explorer-codegen-profiles","version":1,"profiles":[{"name":"default"},{"name":"default"}]}"#
            )
            .unwrap_err()
            .to_string(),
            "profiles[1].name duplicates default"
        );
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
        let root = env::temp_dir().join(format!(
            "gemstone-rs-explorer-post-save-{}",
            std::process::id()
        ));
        let path = root.join("post-save.codegen");
        let _ = fs::remove_dir_all(&root);
        let config = ExplorerConfig {
            read_only: false,
            codegen_root: root.clone(),
            ..ExplorerConfig::default()
        };
        let source = "output = generated/post-save.rs\n";
        let request = HttpRequest {
            method: "POST".to_string(),
            target: "/api/codegen/config/save?config=post-save.codegen".to_string(),
            headers: Vec::new(),
            body: source.to_string(),
        };

        let response = handle_http_request(&request, &config);
        assert_eq!(response.status, 200);
        assert!(response.body.contains(r#""success":true"#));
        assert_eq!(fs::read_to_string(&path).unwrap(), source);
        let _ = fs::remove_dir_all(root);
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
