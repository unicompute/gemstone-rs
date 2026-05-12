use crate::browser;
use std::collections::BTreeMap;
use std::error::Error as StdError;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub type Result<T> = std::result::Result<T, Error>;

pub const DEFAULT_CONFIG_PATH: &str = "gemstone-rs.codegen";
pub const DEFAULT_OUTPUT_PATH: &str = "src/generated/gemstone_wrappers.rs";

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Config {
        path: Option<PathBuf>,
        line: usize,
        message: String,
    },
}

impl Error {
    fn config(path: Option<&Path>, line: usize, message: impl Into<String>) -> Self {
        Self::Config {
            path: path.map(Path::to_path_buf),
            line,
            message: message.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "{err}"),
            Self::Config {
                path,
                line,
                message,
            } => {
                if let Some(path) = path {
                    write!(f, "{}:{line}: {message}", path.display())
                } else {
                    write!(f, "line {line}: {message}")
                }
            }
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::Config { .. } => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub output: PathBuf,
    pub classes: Vec<ClassSpec>,
}

impl Config {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let source = fs::read_to_string(path)?;
        Self::parse(&source, Some(path))
    }

    pub fn parse(source: &str, path: Option<&Path>) -> Result<Self> {
        let base_dir = path
            .and_then(Path::parent)
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let mut output = PathBuf::from(DEFAULT_OUTPUT_PATH);
        let mut classes: BTreeMap<ClassRef, ClassSpec> = BTreeMap::new();

        for (index, raw_line) in source.lines().enumerate() {
            let line_no = index + 1;
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let (key, value) = split_directive(line)
                .ok_or_else(|| Error::config(path, line_no, "expected key=value or key value"))?;
            match key {
                "output" => {
                    if value.is_empty() {
                        return Err(Error::config(path, line_no, "output path is empty"));
                    }
                    output = PathBuf::from(value);
                }
                "class" => {
                    let class_ref = ClassRef::parse(value)
                        .map_err(|message| Error::config(path, line_no, message))?;
                    classes
                        .entry(class_ref.clone())
                        .or_insert_with(|| ClassSpec::new(class_ref));
                }
                "method" => {
                    let method = MethodSpec::parse(value)
                        .map_err(|message| Error::config(path, line_no, message))?;
                    classes
                        .entry(method.class_ref.clone())
                        .or_insert_with(|| ClassSpec::new(method.class_ref.clone()))
                        .methods
                        .push(method);
                }
                other => {
                    return Err(Error::config(
                        path,
                        line_no,
                        format!("unknown directive: {other}"),
                    ));
                }
            }
        }

        if output.is_relative() {
            output = base_dir.join(output);
        }

        Ok(Self {
            output,
            classes: classes.into_values().collect(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassSpec {
    pub class_ref: ClassRef,
    pub methods: Vec<MethodSpec>,
}

impl ClassSpec {
    fn new(class_ref: ClassRef) -> Self {
        Self {
            class_ref,
            methods: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ClassRef {
    pub class_name: String,
    pub dictionary: String,
    pub meta: bool,
}

impl ClassRef {
    pub fn parse(value: &str) -> std::result::Result<Self, String> {
        let mut text = value.trim();
        if text.is_empty() {
            return Err("class reference is empty".to_string());
        }

        let meta = text.ends_with(" class");
        if meta {
            text = text.trim_end_matches(" class").trim_end();
        }

        let (dictionary, class_name) = text
            .split_once(':')
            .map(|(dictionary, class_name)| (dictionary.trim(), class_name.trim()))
            .unwrap_or(("", text));

        if class_name.is_empty() {
            return Err("class name is empty".to_string());
        }

        Ok(Self {
            class_name: class_name.to_string(),
            dictionary: dictionary.to_string(),
            meta,
        })
    }

    pub fn display_name(&self) -> String {
        let class_name = if self.dictionary.is_empty() {
            self.class_name.clone()
        } else {
            format!("{}:{}", self.dictionary, self.class_name)
        };
        if self.meta {
            format!("{class_name} class")
        } else {
            class_name
        }
    }

    fn struct_name(&self) -> String {
        let mut name = rust_type_name(&self.class_name);
        if self.meta {
            name.push_str("Class");
        }
        name
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MethodSpec {
    pub class_ref: ClassRef,
    pub selector: String,
}

impl MethodSpec {
    pub fn parse(value: &str) -> std::result::Result<Self, String> {
        let (class_ref, selector) = value
            .split_once(">>")
            .ok_or_else(|| "method must look like Class>>selector".to_string())?;
        let class_ref = ClassRef::parse(class_ref)?;
        let selector = selector.trim();
        if selector.is_empty() {
            return Err("method selector is empty".to_string());
        }
        Ok(Self {
            class_ref,
            selector: selector.to_string(),
        })
    }

    fn fn_name(&self) -> String {
        rust_fn_name(&self.selector)
    }

    fn arg_count(&self) -> usize {
        self.selector.matches(':').count()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratedCode {
    pub output: PathBuf,
    pub source: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckReport {
    pub output: PathBuf,
    pub exists: bool,
    pub up_to_date: bool,
}

pub fn load_or_sample(path: impl AsRef<Path>) -> Result<Config> {
    let path = path.as_ref();
    if path.exists() {
        Config::from_file(path)
    } else {
        Config::parse(sample_config(), Some(path))
    }
}

pub fn generate(config: &Config) -> GeneratedCode {
    GeneratedCode {
        output: config.output.clone(),
        source: generate_source(config),
    }
}

pub fn generate_to_file(config: &Config) -> Result<GeneratedCode> {
    let generated = generate(config);
    if let Some(parent) = generated.output.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(&generated.output, &generated.source)?;
    Ok(generated)
}

pub fn check(config: &Config) -> Result<CheckReport> {
    let generated = generate(config);
    match fs::read_to_string(&generated.output) {
        Ok(current) => Ok(CheckReport {
            output: generated.output,
            exists: true,
            up_to_date: current == generated.source,
        }),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(CheckReport {
            output: generated.output,
            exists: false,
            up_to_date: false,
        }),
        Err(err) => Err(Error::Io(err)),
    }
}

pub fn sample_config() -> &'static str {
    "# gemstone-rs codegen config\n\
     # Empty dictionary means resolve through the active user's symbol list.\n\
     output = src/generated/gemstone_wrappers.rs\n\
     class = Object\n\
     method = Object>>printString\n\
     method = Object>>class\n"
}

fn generate_source(config: &Config) -> String {
    let mut source = String::new();
    source.push_str("// @generated by gemstone-rs codegen. Do not edit by hand.\n");
    source.push_str("use gemstone_rs::{Oop, Result, Session, Value};\n\n");

    for class in &config.classes {
        let struct_name = class.class_ref.struct_name();
        source.push_str("#[derive(Debug)]\n");
        source.push_str(&format!("pub struct {struct_name}<'a> {{\n"));
        source.push_str("    session: &'a mut Session,\n");
        source.push_str("    oop: Oop,\n");
        source.push_str("}\n\n");
        source.push_str(&format!("impl<'a> {struct_name}<'a> {{\n"));
        source.push_str("    pub fn resolve(session: &'a mut Session) -> Result<Self> {\n");
        source.push_str(&format!(
            "        let oop = session.execute({})?;\n",
            rust_string_literal(&browser::behavior_expr(
                &class.class_ref.class_name,
                class.class_ref.meta,
                &class.class_ref.dictionary,
            ))
        ));
        source.push_str("        Ok(Self { session, oop })\n");
        source.push_str("    }\n\n");
        source.push_str("    pub fn from_oop(session: &'a mut Session, oop: Oop) -> Self {\n");
        source.push_str("        Self { session, oop }\n");
        source.push_str("    }\n\n");
        source.push_str("    pub fn oop(&self) -> Oop {\n");
        source.push_str("        self.oop\n");
        source.push_str("    }\n");

        for method in &class.methods {
            source.push('\n');
            source.push_str(&method_source(method));
        }

        source.push_str("}\n\n");
    }

    source
}

fn method_source(method: &MethodSpec) -> String {
    let mut source = String::new();
    let fn_name = method.fn_name();
    let args: Vec<String> = (0..method.arg_count())
        .map(|index| format!("arg{index}: Oop"))
        .collect();
    let arg_names: Vec<String> = (0..method.arg_count())
        .map(|index| format!("arg{index}"))
        .collect();
    let args_suffix = if args.is_empty() {
        String::new()
    } else {
        format!(", {}", args.join(", "))
    };
    source.push_str(&format!(
        "    pub fn {fn_name}(&mut self{args_suffix}) -> Result<Value> {{\n"
    ));
    source.push_str(&format!(
        "        self.session.perform(self.oop, {}, &[{}])\n",
        rust_string_literal(&method.selector),
        arg_names.join(", ")
    ));
    source.push_str("    }\n");
    source
}

fn split_directive(line: &str) -> Option<(&str, &str)> {
    if let Some((key, value)) = line.split_once('=') {
        return Some((key.trim(), value.trim()));
    }
    let mut parts = line.splitn(2, char::is_whitespace);
    let key = parts.next()?.trim();
    let value = parts.next()?.trim();
    Some((key, value))
}

fn rust_type_name(value: &str) -> String {
    let mut result = String::new();
    let mut capitalize = true;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            if capitalize {
                result.push(ch.to_ascii_uppercase());
                capitalize = false;
            } else {
                result.push(ch);
            }
        } else {
            capitalize = true;
        }
    }
    if result.is_empty() {
        result.push_str("GemStoneObject");
    }
    if result.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        result.insert(0, 'G');
    }
    result
}

fn rust_fn_name(selector: &str) -> String {
    let mut result = String::new();
    let mut previous_was_separator = true;
    for ch in selector.chars() {
        if ch.is_ascii_uppercase() {
            if !result.is_empty() && !previous_was_separator {
                result.push('_');
            }
            result.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if ch.is_ascii_lowercase() || ch.is_ascii_digit() {
            result.push(ch);
            previous_was_separator = false;
        } else if !result.ends_with('_') && !result.is_empty() {
            result.push('_');
            previous_was_separator = true;
        }
    }
    while result.ends_with('_') {
        result.pop();
    }
    if result.is_empty() {
        result.push_str("perform");
    }
    if result.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        result.insert(0, '_');
    }
    if is_rust_keyword(&result) {
        result.push('_');
    }
    result
}

fn rust_string_literal(value: &str) -> String {
    let mut result = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            ch if ch.is_control() => result.push_str(&format!("\\u{{{:x}}}", ch as u32)),
            ch => result.push(ch),
        }
    }
    result.push('"');
    result
}

fn is_rust_keyword(value: &str) -> bool {
    matches!(
        value,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_line_oriented_config() -> Result<()> {
        let config = Config::parse(
            "output = generated.rs\nclass = Object\nmethod = UserGlobals:Order>>findById:\n",
            Some(Path::new("fixtures/gemstone-rs.codegen")),
        )?;

        assert_eq!(config.output, PathBuf::from("fixtures/generated.rs"));
        assert_eq!(config.classes.len(), 2);
        assert_eq!(config.classes[0].class_ref.class_name, "Object");
        assert_eq!(config.classes[1].class_ref.dictionary, "UserGlobals");
        assert_eq!(config.classes[1].methods[0].selector, "findById:");
        Ok(())
    }

    #[test]
    fn parses_class_side_references() {
        let class_ref = ClassRef::parse("UserGlobals:Order class").unwrap();
        assert_eq!(class_ref.dictionary, "UserGlobals");
        assert_eq!(class_ref.class_name, "Order");
        assert!(class_ref.meta);
        assert_eq!(class_ref.display_name(), "UserGlobals:Order class");
        assert_eq!(class_ref.struct_name(), "OrderClass");
    }

    #[test]
    fn sanitizes_selectors_to_rust_function_names() {
        assert_eq!(rust_fn_name("printString"), "print_string");
        assert_eq!(rust_fn_name("at:put:"), "at_put");
        assert_eq!(rust_fn_name("class"), "class");
        assert_eq!(rust_fn_name("type"), "type_");
    }

    #[test]
    fn generates_wrapper_source() -> Result<()> {
        let config = Config::parse(
            "class = Object\nmethod = Object>>printString\nmethod = Object>>at:put:\n",
            None,
        )?;
        let generated = generate(&config);
        assert!(generated.source.contains("pub struct Object<'a>"));
        assert!(generated
            .source
            .contains("pub fn print_string(&mut self) -> Result<Value>"));
        assert!(generated
            .source
            .contains("pub fn at_put(&mut self, arg0: Oop, arg1: Oop)"));
        assert!(generated
            .source
            .contains("self.session.perform(self.oop, \"at:put:\", &[arg0, arg1])"));
        Ok(())
    }

    #[test]
    fn check_reports_missing_output_as_stale() -> Result<()> {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let output = std::env::temp_dir().join(format!("gemstone-rs-codegen-{nonce}.rs"));
        let config = Config {
            output,
            classes: Vec::new(),
        };
        let report = check(&config)?;
        assert!(!report.exists);
        assert!(!report.up_to_date);
        Ok(())
    }
}
