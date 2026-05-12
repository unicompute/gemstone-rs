use gemstone_rs::{
    browser::{Browser, ALL_PROTOCOLS},
    codegen::{self, DEFAULT_CONFIG_PATH},
    Config, Error as GemStoneError, Oop, Session, Value,
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
            let class = session.fetch_class(oop)?;
            let printed = session.perform_oop(oop, "printString", &[])?;
            println!("oop: {}", oop.raw());
            println!("class_oop: {}", class.raw());
            println!("printString: {}", session.fetch_string(printed)?);
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
    }
}

fn login() -> Result<Session, CliError> {
    Ok(Session::login(Config::from_env()?)?)
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

fn print_lines(values: impl IntoIterator<Item = String>) {
    for value in values {
        println!("{value}");
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Command {
    Help,
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
    CodegenInit {
        config: PathBuf,
    },
    CodegenPreview {
        config: PathBuf,
    },
    CodegenCheck {
        config: PathBuf,
    },
    CodegenGenerate {
        config: PathBuf,
    },
}

fn parse_command(args: &[String]) -> Result<Command, CliError> {
    let Some(command) = args.first().map(String::as_str) else {
        return Ok(Command::Help);
    };

    match command {
        "-h" | "--help" | "help" => Ok(Command::Help),
        "eval" => {
            let source = args
                .get(1)
                .ok_or_else(|| CliError::usage("missing Smalltalk source for eval"))?;
            Ok(Command::Eval {
                source: source.clone(),
            })
        }
        "browse" => parse_browse_command(&args[1..]),
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
            Some("check") => Ok(Command::CodegenCheck {
                config: optional_path(args.get(2)),
            }),
            Some("generate") => Ok(Command::CodegenGenerate {
                config: optional_path(args.get(2)),
            }),
            _ => Err(CliError::usage(
                "expected: codegen init|preview|check|generate",
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

fn usage() -> &'static str {
    "usage:
  gemstone-rs eval <smalltalk>
  gemstone-rs browse dictionaries
  gemstone-rs browse classes [dictionary]
  gemstone-rs browse protocols <class> [dictionary] [--meta]
  gemstone-rs browse methods <class> [protocol] [dictionary] [--meta]
  gemstone-rs browse source <class> [selector] [dictionary] [--meta]
  gemstone-rs inspect oop <raw-oop>
  gemstone-rs codegen init [config]
  gemstone-rs codegen preview [config]
  gemstone-rs codegen check [config]
  gemstone-rs codegen generate [config]"
}

#[derive(Debug)]
enum CliError {
    Usage(String),
    GemStone(GemStoneError),
    Codegen(codegen::Error),
    CodegenCheck(String),
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
            Self::CodegenCheck(_) => None,
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
            parse_command(&args(&["codegen", "generate", "demo.codegen"])).unwrap(),
            Command::CodegenGenerate {
                config: PathBuf::from("demo.codegen")
            }
        );
    }
}
