use gemstone_rs::{Config, Error as GemStoneError, Oop, Session, Value};
use std::env;
use std::error::Error as StdError;
use std::fmt;
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
        Command::BrowseClasses => {
            let mut session = login()?;
            let source =
                "System myUserProfile symbolList dictionaries collect: [:dict | dict name]";
            let value = session.eval(source)?;
            print_value(&mut session, value)?;
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
        Command::CodegenCheck => {
            println!("codegen check is planned; no generator is wired into gemstone-rs yet");
            Ok(())
        }
        Command::CodegenGenerate => {
            println!("codegen generate is planned; no generator is wired into gemstone-rs yet");
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

#[derive(Clone, Debug, Eq, PartialEq)]
enum Command {
    Help,
    Eval { source: String },
    BrowseClasses,
    InspectOop { oop: Oop },
    CodegenCheck,
    CodegenGenerate,
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
        "browse" => match args.get(1).map(String::as_str) {
            Some("classes") => Ok(Command::BrowseClasses),
            _ => Err(CliError::usage("expected: browse classes")),
        },
        "inspect" => match (args.get(1).map(String::as_str), args.get(2)) {
            (Some("oop"), Some(raw)) => Ok(Command::InspectOop {
                oop: Oop(parse_u64(raw)?),
            }),
            _ => Err(CliError::usage("expected: inspect oop <raw-oop>")),
        },
        "codegen" => match args.get(1).map(String::as_str) {
            Some("check") => Ok(Command::CodegenCheck),
            Some("generate") => Ok(Command::CodegenGenerate),
            _ => Err(CliError::usage("expected: codegen check|generate")),
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

fn usage() -> &'static str {
    "usage:
  gemstone-rs eval <smalltalk>
  gemstone-rs browse classes
  gemstone-rs inspect oop <raw-oop>
  gemstone-rs codegen check
  gemstone-rs codegen generate"
}

#[derive(Debug)]
enum CliError {
    Usage(String),
    GemStone(GemStoneError),
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
        }
    }
}

impl StdError for CliError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Usage(_) => None,
            Self::GemStone(err) => Some(err),
        }
    }
}

impl From<GemStoneError> for CliError {
    fn from(value: GemStoneError) -> Self {
        Self::GemStone(value)
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
            parse_command(&args(&["codegen", "check"])).unwrap(),
            Command::CodegenCheck
        );
        assert_eq!(
            parse_command(&args(&["codegen", "generate"])).unwrap(),
            Command::CodegenGenerate
        );
    }
}
