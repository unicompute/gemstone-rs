use gemstone_rs::{
    browser::{Browser, ALL_PROTOCOLS},
    codegen::{self, DEFAULT_CONFIG_PATH},
    gci_library_path, BridgeKeyType, Config, Error as GemStoneError, Oop, Session, Value,
    DEFAULT_BRIDGE_ROOT,
};
use std::env;
use std::error::Error as StdError;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(CliError::Usage(message)) => {
            eprintln!("{message}");
            eprintln!();
            eprintln!("{}", usage());
            ExitCode::from(2)
        }
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(1)
        }
    }
}

fn run(args: Vec<String>) -> Result<(), CliError> {
    match parse_command(&args)? {
        Command::Help => {
            println!("{}", usage());
            Ok(())
        }
        Command::Doctor { live, format } => run_doctor(live, format),
        Command::Eval { source } => {
            let mut session = login()?;
            let value = session.eval(&source)?;
            print_value(&mut session, value)?;
            Ok(())
        }
        Command::BrowseDictionaries => {
            let mut session = login()?;
            print_lines(Browser::new(&mut session).dictionaries()?);
            Ok(())
        }
        Command::BrowseClasses { dictionary } => {
            let mut session = login()?;
            print_lines(Browser::new(&mut session).classes(&dictionary)?);
            Ok(())
        }
        Command::BrowseProtocols {
            class_name,
            dictionary,
            meta,
        } => {
            let mut session = login()?;
            println!("{ALL_PROTOCOLS}");
            print_lines(Browser::new(&mut session).protocols(&class_name, meta, &dictionary)?);
            Ok(())
        }
        Command::BrowseMethods {
            class_name,
            protocol,
            dictionary,
            meta,
        } => {
            let mut session = login()?;
            print_lines(Browser::new(&mut session).methods(
                &class_name,
                &protocol,
                meta,
                &dictionary,
            )?);
            Ok(())
        }
        Command::BrowseSource {
            class_name,
            selector,
            dictionary,
            meta,
        } => {
            let mut session = login()?;
            println!(
                "{}",
                Browser::new(&mut session).source(&class_name, &selector, meta, &dictionary)?
            );
            Ok(())
        }
        Command::InspectOop { oop } => {
            let mut session = login()?;
            print_oop_inspection(&mut session, oop)?;
            Ok(())
        }
        Command::BridgeRoot { root } => {
            let mut session = login()?;
            let (name, oop, identity_id) = {
                let bridge_root = session.bridge_root_named(root)?;
                (
                    bridge_root.name().to_string(),
                    bridge_root.oop(),
                    bridge_root.identity_id(),
                )
            };
            println!("root: {name}");
            println!("oop: {}", oop.raw());
            println!("identity_id: {identity_id}");
            println!("identity_map_size: {}", session.identity_map_len());
            Ok(())
        }
        Command::BridgeKeys { root } => {
            let mut session = login()?;
            let keys = {
                let mut bridge_root = session.bridge_root_named(root)?;
                bridge_root.keys()?
            };
            for key in keys {
                println!(
                    "{}\toop={}\tclass_oop={}\tidentity_id={}",
                    key.print_string,
                    key.oop.raw(),
                    key.class_oop.raw(),
                    key.identity_id
                );
            }
            Ok(())
        }
        Command::BridgeGet {
            root,
            key,
            key_type,
        } => {
            let mut session = login()?;
            let value = {
                let mut bridge_root = session.bridge_root_named(root)?;
                bridge_root.get_value_with_key_type(&key, key_type)?
            };
            print_value(&mut session, value)?;
            Ok(())
        }
        Command::BridgeInspect {
            root,
            key,
            key_type,
        } => {
            let mut session = login()?;
            let oop = {
                let mut bridge_root = session.bridge_root_named(root)?;
                bridge_root.get_oop_with_key_type(&key, key_type)?
            };
            print_oop_inspection(&mut session, oop)?;
            Ok(())
        }
        Command::BridgeSampleConfig { mapped_name } => {
            print!("{}", codegen::sample_mapping_config(&mapped_name));
            Ok(())
        }
        Command::CodegenInit { config } => {
            if config.exists() {
                return Err(CliError::CodegenCheck(format!(
                    "{} already exists",
                    config.display()
                )));
            }
            fs::write(&config, codegen::sample_config())?;
            println!("wrote {}", config.display());
            Ok(())
        }
        Command::CodegenPreview { config } => {
            let config = codegen::Config::from_file(config)?;
            print!("{}", codegen::generate(&config).source);
            Ok(())
        }
        Command::CodegenDiff { config } => {
            let config = codegen::Config::from_file(&config)?;
            let report = codegen::diff(&config)?;
            if report.up_to_date {
                println!("codegen ok: {} is up to date", report.output.display());
                Ok(())
            } else {
                print!("{}", report.diff);
                Err(CliError::CodegenCheck(format!(
                    "{} is stale or missing",
                    report.output.display()
                )))
            }
        }
        Command::CodegenCheck { config } => {
            let config_path = config;
            let config = codegen::Config::from_file(&config_path)?;
            let report = codegen::check(&config)?;
            if report.up_to_date {
                println!("codegen ok: {} is up to date", report.output.display());
                Ok(())
            } else {
                Err(CliError::CodegenCheck(format!(
                    "{} is stale or missing; run `gemstone-rs codegen generate {}`",
                    report.output.display(),
                    config_path.display()
                )))
            }
        }
        Command::CodegenGenerate { config } => {
            let config = codegen::Config::from_file(config)?;
            let generated = codegen::generate_to_file(&config)?;
            println!("generated {}", generated.output.display());
            Ok(())
        }
        Command::CodegenDiscover { config, classes } => {
            let class_refs: Vec<codegen::ClassRef> = classes
                .iter()
                .map(|class| codegen::ClassRef::parse(class))
                .collect::<Result<_, _>>()
                .map_err(CliError::usage)?;
            let mut session = login()?;
            let discovered =
                codegen::discover(&mut session, default_codegen_output(&config), &class_refs)?;
            codegen::write_config(&config, &discovered)?;
            println!(
                "wrote {} with {} classes",
                config.display(),
                discovered.classes.len()
            );
            Ok(())
        }
        Command::CodegenDiscoverMapping {
            config,
            mapped_name,
            class_name,
        } => {
            let class_ref = codegen::ClassRef::parse(&class_name).map_err(CliError::usage)?;
            let mut session = login()?;
            let discovered = codegen::discover_mapping(
                &mut session,
                default_codegen_output(&config),
                &mapped_name,
                &class_ref,
            )?;
            codegen::write_config(&config, &discovered)?;
            println!(
                "wrote {} with mapping {} from {}",
                config.display(),
                discovered
                    .mapped
                    .first()
                    .map(|mapped| mapped.name.as_str())
                    .unwrap_or("<none>"),
                class_ref.display_name()
            );
            Ok(())
        }
    }
}

fn login() -> Result<Session, CliError> {
    Ok(Session::login(Config::from_env()?)?)
}

fn run_doctor(live: bool, format: OutputFormat) -> Result<(), CliError> {
    if format == OutputFormat::Json {
        return run_doctor_json(live);
    }

    let mut ok = true;
    println!("gemstone-rs doctor");
    println!("environment:");
    print_env_value("GS_LIB_PATH", false);
    print_env_value("GS_LIB", false);
    print_env_value("GEMSTONE", false);
    print_env_value("GS_STONE", false);
    print_env_value("GS_STONE_NAME", false);
    print_env_value("GS_HOST", false);
    print_env_value("GS_NETLDI", false);
    print_env_value("GS_GEM_SERVICE", false);
    print_env_value("GS_USERNAME", false);
    print_env_value("GS_PASSWORD", true);
    print_env_value("GS_HOST_USERNAME", false);
    print_env_value("GS_HOST_PASSWORD", true);

    let config = match Config::from_env() {
        Ok(config) => {
            println!("config: ok");
            println!("  stone_nrs: {}", config.stone_nrs());
            Some(config)
        }
        Err(err) => {
            ok = false;
            println!("config: error: {err}");
            None
        }
    };

    if let Some(config) = config {
        match gci_library_path(&config) {
            Ok(path) => println!("gci_library: ok: {}", path.display()),
            Err(err) => {
                ok = false;
                println!("gci_library: error: {err}");
            }
        }

        if live {
            match Session::login(config).and_then(|mut session| session.eval("3 + 4")) {
                Ok(Value::SmallInt(7)) => println!("live: ok: 3 + 4 == 7"),
                Ok(value) => {
                    ok = false;
                    println!("live: error: expected 7, got {value:?}");
                }
                Err(err) => {
                    ok = false;
                    println!("live: error: {err}");
                }
            }
        } else {
            println!("live: skipped; pass --live to login and evaluate 3 + 4");
        }
    } else if live {
        println!("live: skipped because config is incomplete");
    }

    if ok {
        Ok(())
    } else {
        Err(CliError::Doctor(
            "doctor found setup problems; see report above".to_string(),
        ))
    }
}

fn run_doctor_json(live: bool) -> Result<(), CliError> {
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

    println!(
        r#"{{"success":{},"environment":{},"config":{},"gciLibrary":{},"live":{}}}"#,
        ok,
        environment_json(),
        config_json,
        gci_json,
        live_json
    );

    if ok {
        Ok(())
    } else {
        Err(CliError::Doctor(
            "doctor found setup problems; see report above".to_string(),
        ))
    }
}

fn print_env_value(name: &str, secret: bool) {
    match env::var(name) {
        Ok(value) if secret && !value.is_empty() => println!("  {name}: <set>"),
        Ok(value) if value.is_empty() => println!("  {name}: <empty>"),
        Ok(value) => println!("  {name}: {value}"),
        Err(_) => println!("  {name}: <unset>"),
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

fn print_value(session: &mut Session, value: Value) -> Result<(), CliError> {
    match value {
        Value::Nil => println!("nil"),
        Value::Bool(value) => println!("{value}"),
        Value::SmallInt(value) => println!("{value}"),
        Value::Char(value) => println!("{value}"),
        Value::String(value) => println!("{value}"),
        Value::Oop(oop) => {
            let printed = session.perform_oop(oop, "printString", &[])?;
            println!("{}", session.fetch_string(printed)?);
        }
    }
    Ok(())
}

fn print_oop_inspection(session: &mut Session, oop: Oop) -> Result<(), CliError> {
    let class = session.fetch_class(oop)?;
    let printed = session.perform_oop(oop, "printString", &[])?;
    println!("oop: {}", oop.raw());
    println!("class_oop: {}", class.raw());
    println!("printString: {}", session.fetch_string(printed)?);
    Ok(())
}

fn print_lines(values: impl IntoIterator<Item = String>) {
    for value in values {
        println!("{value}");
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Command {
    Help,
    Doctor {
        live: bool,
        format: OutputFormat,
    },
    Eval {
        source: String,
    },
    BrowseDictionaries,
    BrowseClasses {
        dictionary: String,
    },
    BrowseProtocols {
        class_name: String,
        dictionary: String,
        meta: bool,
    },
    BrowseMethods {
        class_name: String,
        protocol: String,
        dictionary: String,
        meta: bool,
    },
    BrowseSource {
        class_name: String,
        selector: String,
        dictionary: String,
        meta: bool,
    },
    InspectOop {
        oop: Oop,
    },
    BridgeRoot {
        root: String,
    },
    BridgeKeys {
        root: String,
    },
    BridgeGet {
        root: String,
        key: String,
        key_type: BridgeKeyType,
    },
    BridgeInspect {
        root: String,
        key: String,
        key_type: BridgeKeyType,
    },
    BridgeSampleConfig {
        mapped_name: String,
    },
    CodegenInit {
        config: PathBuf,
    },
    CodegenPreview {
        config: PathBuf,
    },
    CodegenDiff {
        config: PathBuf,
    },
    CodegenCheck {
        config: PathBuf,
    },
    CodegenGenerate {
        config: PathBuf,
    },
    CodegenDiscover {
        config: PathBuf,
        classes: Vec<String>,
    },
    CodegenDiscoverMapping {
        config: PathBuf,
        mapped_name: String,
        class_name: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OutputFormat {
    Human,
    Json,
}

fn parse_command(args: &[String]) -> Result<Command, CliError> {
    let Some(command) = args.first().map(String::as_str) else {
        return Ok(Command::Help);
    };

    match command {
        "-h" | "--help" | "help" => Ok(Command::Help),
        "doctor" => parse_doctor_command(&args[1..]),
        "eval" => {
            let source = args
                .get(1)
                .ok_or_else(|| CliError::usage("missing Smalltalk source for eval"))?;
            Ok(Command::Eval {
                source: source.clone(),
            })
        }
        "browse" => parse_browse_command(&args[1..]),
        "bridge" => parse_bridge_command(&args[1..]),
        "inspect" => match (args.get(1).map(String::as_str), args.get(2)) {
            (Some("oop"), Some(raw)) => Ok(Command::InspectOop {
                oop: Oop(parse_u64(raw)?),
            }),
            _ => Err(CliError::usage("expected: inspect oop <raw-oop>")),
        },
        "codegen" => match args.get(1).map(String::as_str) {
            Some("init") => Ok(Command::CodegenInit {
                config: optional_path(args.get(2)),
            }),
            Some("preview") => Ok(Command::CodegenPreview {
                config: optional_path(args.get(2)),
            }),
            Some("diff") => Ok(Command::CodegenDiff {
                config: optional_path(args.get(2)),
            }),
            Some("check") => Ok(Command::CodegenCheck {
                config: optional_path(args.get(2)),
            }),
            Some("generate") => Ok(Command::CodegenGenerate {
                config: optional_path(args.get(2)),
            }),
            Some("discover") => Ok(Command::CodegenDiscover {
                config: optional_path(args.get(2)),
                classes: args.iter().skip(3).cloned().collect(),
            }),
            Some("discover-mapping") => Ok(Command::CodegenDiscoverMapping {
                config: optional_path(args.get(2)),
                mapped_name: args
                    .get(3)
                    .cloned()
                    .ok_or_else(|| CliError::usage("missing mapped struct name"))?,
                class_name: args
                    .get(4)
                    .cloned()
                    .ok_or_else(|| CliError::usage("missing GemStone class"))?,
            }),
            _ => Err(CliError::usage(
                "expected: codegen init|preview|diff|check|generate|discover|discover-mapping",
            )),
        },
        _ => Err(CliError::usage(format!("unknown command: {command}"))),
    }
}

fn parse_u64(value: &str) -> Result<u64, CliError> {
    if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        u64::from_str_radix(hex, 16)
    } else {
        value.parse()
    }
    .map_err(|_| CliError::usage(format!("invalid OOP: {value}")))
}

fn parse_doctor_command(args: &[String]) -> Result<Command, CliError> {
    let mut live = false;
    let mut format = OutputFormat::Human;
    for arg in args {
        match arg.as_str() {
            "--live" => live = true,
            "--json" => format = OutputFormat::Json,
            other => return Err(CliError::usage(format!("unknown doctor option: {other}"))),
        }
    }
    Ok(Command::Doctor { live, format })
}

fn parse_browse_command(args: &[String]) -> Result<Command, CliError> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err(CliError::usage(
            "expected: browse dictionaries|classes|protocols|methods|source",
        ));
    };
    let (values, meta) = split_meta_flag(&args[1..]);
    match command {
        "dictionaries" => Ok(Command::BrowseDictionaries),
        "classes" => Ok(Command::BrowseClasses {
            dictionary: values
                .first()
                .cloned()
                .unwrap_or_else(|| "UserGlobals".to_string()),
        }),
        "protocols" => Ok(Command::BrowseProtocols {
            class_name: required_value(&values, 0, "missing class for browse protocols")?,
            dictionary: values.get(1).cloned().unwrap_or_default(),
            meta,
        }),
        "methods" => Ok(Command::BrowseMethods {
            class_name: required_value(&values, 0, "missing class for browse methods")?,
            protocol: values
                .get(1)
                .cloned()
                .unwrap_or_else(|| ALL_PROTOCOLS.to_string()),
            dictionary: values.get(2).cloned().unwrap_or_default(),
            meta,
        }),
        "source" => Ok(Command::BrowseSource {
            class_name: required_value(&values, 0, "missing class for browse source")?,
            selector: values.get(1).cloned().unwrap_or_default(),
            dictionary: values.get(2).cloned().unwrap_or_default(),
            meta,
        }),
        _ => Err(CliError::usage(
            "expected: browse dictionaries|classes|protocols|methods|source",
        )),
    }
}

fn parse_bridge_command(args: &[String]) -> Result<Command, CliError> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err(CliError::usage(
            "expected: bridge root|keys|get|inspect|sample-config",
        ));
    };
    match command {
        "root" => {
            let options = parse_bridge_options(&args[1..])?;
            Ok(Command::BridgeRoot { root: options.root })
        }
        "keys" => {
            let options = parse_bridge_options(&args[1..])?;
            Ok(Command::BridgeKeys { root: options.root })
        }
        "get" => {
            let key = args
                .get(1)
                .cloned()
                .ok_or_else(|| CliError::usage("missing key for bridge get"))?;
            let options = parse_bridge_options(&args[2..])?;
            Ok(Command::BridgeGet {
                root: options.root,
                key,
                key_type: options.key_type,
            })
        }
        "inspect" => {
            let key = args
                .get(1)
                .cloned()
                .ok_or_else(|| CliError::usage("missing key for bridge inspect"))?;
            let options = parse_bridge_options(&args[2..])?;
            Ok(Command::BridgeInspect {
                root: options.root,
                key,
                key_type: options.key_type,
            })
        }
        "sample-config" => Ok(Command::BridgeSampleConfig {
            mapped_name: args
                .get(1)
                .cloned()
                .unwrap_or_else(|| "BookingDraft".to_string()),
        }),
        _ => Err(CliError::usage(
            "expected: bridge root|keys|get|inspect|sample-config",
        )),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BridgeOptions {
    root: String,
    key_type: BridgeKeyType,
}

fn parse_bridge_options(args: &[String]) -> Result<BridgeOptions, CliError> {
    let mut root = DEFAULT_BRIDGE_ROOT.to_string();
    let mut key_type = BridgeKeyType::String;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--root" => {
                index += 1;
                root = args
                    .get(index)
                    .cloned()
                    .ok_or_else(|| CliError::usage("missing value for --root"))?;
            }
            "--symbol" => key_type = BridgeKeyType::Symbol,
            "--string" => key_type = BridgeKeyType::String,
            "--key-type" => {
                index += 1;
                key_type = parse_bridge_key_type(
                    args.get(index)
                        .ok_or_else(|| CliError::usage("missing value for --key-type"))?,
                )?;
            }
            other => {
                return Err(CliError::usage(format!("unknown bridge option: {other}")));
            }
        }
        index += 1;
    }
    Ok(BridgeOptions { root, key_type })
}

fn parse_bridge_key_type(value: &str) -> Result<BridgeKeyType, CliError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "string" => Ok(BridgeKeyType::String),
        "symbol" => Ok(BridgeKeyType::Symbol),
        _ => Err(CliError::usage(format!(
            "expected bridge key type String or Symbol, got {value}"
        ))),
    }
}

fn split_meta_flag(args: &[String]) -> (Vec<String>, bool) {
    let mut meta = false;
    let mut values = Vec::new();
    for value in args {
        if value == "--meta" || value == "--class-side" {
            meta = true;
        } else {
            values.push(value.clone());
        }
    }
    (values, meta)
}

fn required_value(
    values: &[String],
    index: usize,
    message: &'static str,
) -> Result<String, CliError> {
    values
        .get(index)
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .ok_or_else(|| CliError::usage(message))
}

fn optional_path(value: Option<&String>) -> PathBuf {
    value
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_PATH))
}

fn default_codegen_output(config: &std::path::Path) -> PathBuf {
    config
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("generated/gemstone_wrappers.rs")
}

fn usage() -> &'static str {
    "usage:
  gemstone-rs doctor [--live] [--json]
  gemstone-rs eval <smalltalk>
  gemstone-rs browse dictionaries
  gemstone-rs browse classes [dictionary]
  gemstone-rs browse protocols <class> [dictionary] [--meta]
  gemstone-rs browse methods <class> [protocol] [dictionary] [--meta]
  gemstone-rs browse source <class> [selector] [dictionary] [--meta]
  gemstone-rs inspect oop <raw-oop>
  gemstone-rs bridge root [--root <name>]
  gemstone-rs bridge keys [--root <name>]
  gemstone-rs bridge get <key> [--symbol|--string|--key-type String|Symbol] [--root <name>]
  gemstone-rs bridge inspect <key> [--symbol|--string|--key-type String|Symbol] [--root <name>]
  gemstone-rs bridge sample-config [mapped-name]
  gemstone-rs codegen init [config]
  gemstone-rs codegen preview [config]
  gemstone-rs codegen diff [config]
  gemstone-rs codegen check [config]
  gemstone-rs codegen generate [config]
  gemstone-rs codegen discover [config] [class ...]
  gemstone-rs codegen discover-mapping [config] <mapped-name> <class>"
}

#[derive(Debug)]
enum CliError {
    Usage(String),
    GemStone(GemStoneError),
    Codegen(codegen::Error),
    CodegenCheck(String),
    Doctor(String),
    Io(std::io::Error),
}

impl CliError {
    fn usage(message: impl Into<String>) -> Self {
        Self::Usage(message.into())
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => write!(f, "{message}"),
            Self::GemStone(err) => write!(f, "{err}"),
            Self::Codegen(err) => write!(f, "{err}"),
            Self::CodegenCheck(message) => write!(f, "{message}"),
            Self::Doctor(message) => write!(f, "{message}"),
            Self::Io(err) => write!(f, "{err}"),
        }
    }
}

impl StdError for CliError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Usage(_) => None,
            Self::GemStone(err) => Some(err),
            Self::Codegen(err) => Some(err),
            Self::CodegenCheck(_) | Self::Doctor(_) => None,
            Self::Io(err) => Some(err),
        }
    }
}

impl From<GemStoneError> for CliError {
    fn from(value: GemStoneError) -> Self {
        Self::GemStone(value)
    }
}

impl From<codegen::Error> for CliError {
    fn from(value: codegen::Error) -> Self {
        Self::Codegen(value)
    }
}

impl From<std::io::Error> for CliError {
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
    fn parses_help_without_live_gemstone() {
        assert_eq!(parse_command(&args(&[])).unwrap(), Command::Help);
        assert_eq!(parse_command(&args(&["--help"])).unwrap(), Command::Help);
    }

    #[test]
    fn parses_doctor_command() {
        assert_eq!(
            parse_command(&args(&["doctor"])).unwrap(),
            Command::Doctor {
                live: false,
                format: OutputFormat::Human,
            }
        );
        assert_eq!(
            parse_command(&args(&["doctor", "--live"])).unwrap(),
            Command::Doctor {
                live: true,
                format: OutputFormat::Human,
            }
        );
        assert_eq!(
            parse_command(&args(&["doctor", "--json", "--live"])).unwrap(),
            Command::Doctor {
                live: true,
                format: OutputFormat::Json,
            }
        );
    }

    #[test]
    fn parses_eval_command() {
        assert_eq!(
            parse_command(&args(&["eval", "3 + 4"])).unwrap(),
            Command::Eval {
                source: "3 + 4".to_string()
            }
        );
    }

    #[test]
    fn parses_browse_commands() {
        assert_eq!(
            parse_command(&args(&["browse", "dictionaries"])).unwrap(),
            Command::BrowseDictionaries
        );
        assert_eq!(
            parse_command(&args(&["browse", "classes"])).unwrap(),
            Command::BrowseClasses {
                dictionary: "UserGlobals".to_string()
            }
        );
        assert_eq!(
            parse_command(&args(&["browse", "protocols", "Object"])).unwrap(),
            Command::BrowseProtocols {
                class_name: "Object".to_string(),
                dictionary: String::new(),
                meta: false,
            }
        );
        assert_eq!(
            parse_command(&args(&[
                "browse",
                "methods",
                "Object",
                "-- all --",
                "UserGlobals",
                "--meta"
            ]))
            .unwrap(),
            Command::BrowseMethods {
                class_name: "Object".to_string(),
                protocol: "-- all --".to_string(),
                dictionary: "UserGlobals".to_string(),
                meta: true,
            }
        );
    }

    #[test]
    fn parses_inspect_oop_decimal_and_hex() {
        assert_eq!(
            parse_command(&args(&["inspect", "oop", "20"])).unwrap(),
            Command::InspectOop { oop: Oop(20) }
        );
        assert_eq!(
            parse_command(&args(&["inspect", "oop", "0x14"])).unwrap(),
            Command::InspectOop { oop: Oop(20) }
        );
    }

    #[test]
    fn parses_bridge_commands() {
        assert_eq!(
            parse_command(&args(&["bridge", "root"])).unwrap(),
            Command::BridgeRoot {
                root: DEFAULT_BRIDGE_ROOT.to_string()
            }
        );
        assert_eq!(
            parse_command(&args(&["bridge", "root", "--root", "DemoRoot"])).unwrap(),
            Command::BridgeRoot {
                root: "DemoRoot".to_string()
            }
        );
        assert_eq!(
            parse_command(&args(&["bridge", "keys", "--root", "DemoRoot"])).unwrap(),
            Command::BridgeKeys {
                root: "DemoRoot".to_string()
            }
        );
        assert_eq!(
            parse_command(&args(&["bridge", "get", "BookingDraft", "--symbol"])).unwrap(),
            Command::BridgeGet {
                root: DEFAULT_BRIDGE_ROOT.to_string(),
                key: "BookingDraft".to_string(),
                key_type: BridgeKeyType::Symbol,
            }
        );
        assert_eq!(
            parse_command(&args(&[
                "bridge",
                "inspect",
                "BookingDraft",
                "--key-type",
                "String",
                "--root",
                "DemoRoot"
            ]))
            .unwrap(),
            Command::BridgeInspect {
                root: "DemoRoot".to_string(),
                key: "BookingDraft".to_string(),
                key_type: BridgeKeyType::String,
            }
        );
        assert_eq!(
            parse_command(&args(&["bridge", "sample-config", "CustomerDraft"])).unwrap(),
            Command::BridgeSampleConfig {
                mapped_name: "CustomerDraft".to_string()
            }
        );
    }

    #[test]
    fn parses_codegen_commands() {
        assert_eq!(
            parse_command(&args(&["codegen", "init"])).unwrap(),
            Command::CodegenInit {
                config: PathBuf::from(DEFAULT_CONFIG_PATH)
            }
        );
        assert_eq!(
            parse_command(&args(&["codegen", "preview", "demo.codegen"])).unwrap(),
            Command::CodegenPreview {
                config: PathBuf::from("demo.codegen")
            }
        );
        assert_eq!(
            parse_command(&args(&["codegen", "check"])).unwrap(),
            Command::CodegenCheck {
                config: PathBuf::from(DEFAULT_CONFIG_PATH)
            }
        );
        assert_eq!(
            parse_command(&args(&["codegen", "diff", "demo.codegen"])).unwrap(),
            Command::CodegenDiff {
                config: PathBuf::from("demo.codegen")
            }
        );
        assert_eq!(
            parse_command(&args(&["codegen", "generate", "demo.codegen"])).unwrap(),
            Command::CodegenGenerate {
                config: PathBuf::from("demo.codegen")
            }
        );
        assert_eq!(
            parse_command(&args(&["codegen", "discover", "demo.codegen", "Object"])).unwrap(),
            Command::CodegenDiscover {
                config: PathBuf::from("demo.codegen"),
                classes: vec!["Object".to_string()]
            }
        );
        assert_eq!(
            parse_command(&args(&[
                "codegen",
                "discover-mapping",
                "mapping.codegen",
                "BookingDraft",
                "UserGlobals:Booking"
            ]))
            .unwrap(),
            Command::CodegenDiscoverMapping {
                config: PathBuf::from("mapping.codegen"),
                mapped_name: "BookingDraft".to_string(),
                class_name: "UserGlobals:Booking".to_string()
            }
        );
    }
}
