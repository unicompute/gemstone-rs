use crate::{browser, browser::Browser, Session};
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
    GemStone(crate::Error),
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
            Self::GemStone(err) => write!(f, "{err}"),
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
            Self::GemStone(err) => Some(err),
            Self::Config { .. } => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<crate::Error> for Error {
    fn from(value: crate::Error) -> Self {
        Self::GemStone(value)
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
    pub args: Vec<String>,
    pub return_type: ReturnType,
    pub doc: Option<String>,
}

impl MethodSpec {
    pub fn parse(value: &str) -> std::result::Result<Self, String> {
        let mut parts = value.split('|').map(str::trim);
        let head = parts
            .next()
            .ok_or_else(|| "method must look like Class>>selector".to_string())?;
        let (class_ref, selector) = head
            .split_once(">>")
            .ok_or_else(|| "method must look like Class>>selector".to_string())?;
        let class_ref = ClassRef::parse(class_ref)?;
        let selector = selector.trim();
        if selector.is_empty() {
            return Err("method selector is empty".to_string());
        }
        let mut args = Vec::new();
        let mut return_type = ReturnType::Value;
        let mut doc = None;
        for option in parts {
            let Some((key, value)) = option.split_once('=') else {
                return Err(format!("method option must look like key=value: {option}"));
            };
            match key.trim() {
                "args" => {
                    args = value
                        .split(',')
                        .map(str::trim)
                        .filter(|arg| !arg.is_empty())
                        .map(str::to_string)
                        .collect();
                }
                "return" => return_type = ReturnType::parse(value.trim())?,
                "doc" => doc = Some(value.trim().to_string()),
                other => return Err(format!("unknown method option: {other}")),
            }
        }
        let selector_arg_count = selector.matches(':').count();
        if !args.is_empty() && args.len() != selector_arg_count {
            return Err(format!(
                "selector {selector} expects {selector_arg_count} arguments, got {} names",
                args.len()
            ));
        }
        Ok(Self {
            class_ref,
            selector: selector.to_string(),
            args,
            return_type,
            doc,
        })
    }

    fn fn_name(&self) -> String {
        rust_fn_name(&self.selector)
    }

    fn arg_count(&self) -> usize {
        self.selector.matches(':').count()
    }

    fn arg_names(&self) -> Vec<String> {
        if self.args.is_empty() {
            (0..self.arg_count())
                .map(|index| format!("arg{index}"))
                .collect()
        } else {
            self.args.iter().map(|arg| rust_fn_name(arg)).collect()
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReturnType {
    Value,
    String,
    SmallInt,
    Bool,
    Oop,
}

impl ReturnType {
    fn parse(value: &str) -> std::result::Result<Self, String> {
        match value {
            "" | "Value" | "value" => Ok(Self::Value),
            "String" | "string" => Ok(Self::String),
            "SmallInt" | "smallInt" | "smallint" | "i64" => Ok(Self::SmallInt),
            "Bool" | "Boolean" | "bool" | "boolean" => Ok(Self::Bool),
            "Oop" | "OOP" | "oop" => Ok(Self::Oop),
            other => Err(format!("unsupported return type: {other}")),
        }
    }

    fn config_name(&self) -> &'static str {
        match self {
            Self::Value => "Value",
            Self::String => "String",
            Self::SmallInt => "SmallInt",
            Self::Bool => "Bool",
            Self::Oop => "Oop",
        }
    }

    fn rust_type(&self) -> &'static str {
        match self {
            Self::Value => "Value",
            Self::String => "String",
            Self::SmallInt => "i64",
            Self::Bool => "bool",
            Self::Oop => "Oop",
        }
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiffReport {
    pub output: PathBuf,
    pub exists: bool,
    pub up_to_date: bool,
    pub diff: String,
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

pub fn write_config(path: impl AsRef<Path>, config: &Config) -> Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(path, config_source(config))?;
    Ok(())
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

pub fn diff(config: &Config) -> Result<DiffReport> {
    let generated = generate(config);
    match fs::read_to_string(&generated.output) {
        Ok(current) => {
            let up_to_date = current == generated.source;
            let diff = if up_to_date {
                String::new()
            } else {
                simple_diff(&generated.output, &current, &generated.source)
            };
            Ok(DiffReport {
                output: generated.output,
                exists: true,
                up_to_date,
                diff,
            })
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(DiffReport {
            output: generated.output.clone(),
            exists: false,
            up_to_date: false,
            diff: simple_diff(&generated.output, "", &generated.source),
        }),
        Err(err) => Err(Error::Io(err)),
    }
}

pub fn discover(session: &mut Session, output: PathBuf, classes: &[ClassRef]) -> Result<Config> {
    let classes = if classes.is_empty() {
        vec![ClassRef::parse("Object").map_err(|message| Error::config(None, 0, message))?]
    } else {
        classes.to_vec()
    };
    let mut browser = Browser::new(session);
    let mut specs = Vec::new();
    for class_ref in classes {
        let selectors = browser.methods(
            &class_ref.class_name,
            browser::ALL_PROTOCOLS,
            class_ref.meta,
            &class_ref.dictionary,
        )?;
        let mut spec = ClassSpec::new(class_ref.clone());
        for selector in selectors {
            let source = browser
                .source(
                    &class_ref.class_name,
                    &selector,
                    class_ref.meta,
                    &class_ref.dictionary,
                )
                .unwrap_or_default();
            spec.methods.push(MethodSpec {
                class_ref: class_ref.clone(),
                selector,
                args: Vec::new(),
                return_type: ReturnType::Value,
                doc: first_source_line(&source),
            });
        }
        specs.push(spec);
    }
    Ok(Config {
        output,
        classes: specs,
    })
}

pub fn config_source(config: &Config) -> String {
    let mut source = String::new();
    source.push_str("# gemstone-rs codegen config\n");
    source.push_str("# Empty dictionary means resolve through the active user's symbol list.\n");
    source.push_str(&format!("output = {}\n", config.output.display()));
    for class in &config.classes {
        source.push_str(&format!("class = {}\n", class.class_ref.display_name()));
        for method in &class.methods {
            source.push_str("method = ");
            source.push_str(&method.class_ref.display_name());
            source.push_str(">>");
            source.push_str(&method.selector);
            if !method.args.is_empty() {
                source.push_str(" | args=");
                source.push_str(&method.args.join(","));
            }
            if method.return_type != ReturnType::Value {
                source.push_str(" | return=");
                source.push_str(method.return_type.config_name());
            }
            if let Some(doc) = method.doc.as_deref().filter(|doc| !doc.is_empty()) {
                source.push_str(" | doc=");
                source.push_str(&doc.replace('\n', " "));
            }
            source.push('\n');
        }
    }
    source
}

pub fn sample_config() -> &'static str {
    "# gemstone-rs codegen config\n\
     # Empty dictionary means resolve through the active user's symbol list.\n\
     output = src/generated/gemstone_wrappers.rs\n\
     class = Object\n\
     method = Object>>printString | return=String | doc=Return the receiver printString.\n\
     method = Object>>class\n"
}

fn generate_source(config: &Config) -> String {
    let mut source = String::new();
    source.push_str("// @generated by gemstone-rs codegen. Do not edit by hand.\n");
    source.push_str("use gemstone_rs::{Error, Oop, Result, Session, Value};\n\n");

    for class in &config.classes {
        let struct_name = class.class_ref.struct_name();
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
    let arg_names = method.arg_names();
    let args: Vec<String> = arg_names.iter().map(|arg| format!("{arg}: Oop")).collect();
    let args_suffix = if args.is_empty() {
        String::new()
    } else {
        format!(", {}", args.join(", "))
    };
    if let Some(doc) = method.doc.as_deref().filter(|doc| !doc.is_empty()) {
        source.push_str(&format!("    /// {}\n", escape_doc(doc)));
    }
    source.push_str(&format!(
        "    pub fn {fn_name}(&mut self{args_suffix}) -> Result<{}> {{\n",
        method.return_type.rust_type()
    ));
    source.push_str(&format!(
        "        let value = self.session.perform(self.oop, {}, &[{}])?;\n",
        rust_string_literal(&method.selector),
        arg_names.join(", ")
    ));
    source.push_str(&return_conversion(&method.return_type));
    source.push_str("    }\n");
    source
}

fn return_conversion(return_type: &ReturnType) -> String {
    match return_type {
        ReturnType::Value => "        Ok(value)\n".to_string(),
        ReturnType::String => typed_match(
            "String",
            &[
                "            Value::String(value) => Ok(value),",
                "            Value::Oop(oop) => self.session.fetch_string(oop),",
            ],
        ),
        ReturnType::SmallInt => typed_match(
            "SmallInt",
            &["            Value::SmallInt(value) => Ok(value),"],
        ),
        ReturnType::Bool => typed_match("Bool", &["            Value::Bool(value) => Ok(value),"]),
        ReturnType::Oop => typed_match("Oop", &["            Value::Oop(oop) => Ok(oop),"]),
    }
}

fn typed_match(expected: &'static str, arms: &[&str]) -> String {
    let mut source = String::from("        match value {\n");
    for arm in arms {
        source.push_str(arm);
        source.push('\n');
    }
    source.push_str("            other => Err(Error::UnexpectedType {\n");
    source.push_str(&format!("                expected: {expected:?},\n"));
    source.push_str("                actual: format!(\"{other:?}\"),\n");
    source.push_str("            }),\n");
    source.push_str("        }\n");
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

fn first_source_line(source: &str) -> Option<String> {
    source
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(|line| line.chars().take(120).collect())
}

fn simple_diff(path: &Path, current: &str, generated: &str) -> String {
    let mut diff = String::new();
    diff.push_str(&format!("--- {}\n", path.display()));
    diff.push_str(&format!("+++ {} (generated)\n", path.display()));
    for line in current.lines() {
        diff.push('-');
        diff.push_str(line);
        diff.push('\n');
    }
    for line in generated.lines() {
        diff.push('+');
        diff.push_str(line);
        diff.push('\n');
    }
    diff
}

fn escape_doc(value: &str) -> String {
    value.replace(['\r', '\n'], " ")
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
            "output = generated.rs\nclass = Object\nmethod = UserGlobals:Order>>findById: | args=id | return=Oop | doc=Find an order.\n",
            Some(Path::new("fixtures/gemstone-rs.codegen")),
        )?;

        assert_eq!(config.output, PathBuf::from("fixtures/generated.rs"));
        assert_eq!(config.classes.len(), 2);
        assert_eq!(config.classes[0].class_ref.class_name, "Object");
        assert_eq!(config.classes[1].class_ref.dictionary, "UserGlobals");
        assert_eq!(config.classes[1].methods[0].selector, "findById:");
        assert_eq!(config.classes[1].methods[0].args, vec!["id"]);
        assert_eq!(config.classes[1].methods[0].return_type, ReturnType::Oop);
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
            "class = Object\nmethod = Object>>printString | return=String | doc=Print the receiver.\nmethod = Object>>at:put: | args=key,value\n",
            None,
        )?;
        let generated = generate(&config);
        assert!(generated.source.contains("pub struct Object<'a>"));
        assert!(generated.source.contains("/// Print the receiver."));
        assert!(generated
            .source
            .contains("pub fn print_string(&mut self) -> Result<String>"));
        assert!(generated
            .source
            .contains("pub fn at_put(&mut self, key: Oop, value: Oop)"));
        assert!(generated
            .source
            .contains("self.session.perform(self.oop, \"at:put:\", &[key, value])"));
        Ok(())
    }

    #[test]
    fn creates_config_source_and_diff() -> Result<()> {
        let config = Config::parse("class = Object\nmethod = Object>>class\n", None)?;
        let source = config_source(&config);
        assert!(source.contains("method = Object>>class"));
        let report = diff(&Config {
            output: std::env::temp_dir().join("gemstone-rs-missing-diff.rs"),
            classes: config.classes,
        })?;
        assert!(!report.up_to_date);
        assert!(report.diff.contains("+++"));
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
