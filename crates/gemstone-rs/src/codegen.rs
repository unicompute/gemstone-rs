use crate::{browser, browser::Browser, BridgeKeyType, BridgeValue, Session};
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
    pub mapped: Vec<MappedSpec>,
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
        let mut mapped: BTreeMap<String, MappedSpec> = BTreeMap::new();

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
                "mapped" => {
                    let spec = MappedSpec::parse(value)
                        .map_err(|message| Error::config(path, line_no, message))?;
                    mapped.entry(spec.name.clone()).or_insert(spec);
                }
                "field" => {
                    let field = FieldSpec::parse(value)
                        .map_err(|message| Error::config(path, line_no, message))?;
                    mapped
                        .entry(field.mapped_name.clone())
                        .or_insert_with(|| MappedSpec::new(field.mapped_name.clone()))
                        .fields
                        .push(field);
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
            mapped: mapped.into_values().collect(),
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
pub struct MappedSpec {
    pub name: String,
    pub fields: Vec<FieldSpec>,
    pub doc: Option<String>,
}

impl MappedSpec {
    fn new(name: String) -> Self {
        Self {
            name,
            fields: Vec::new(),
            doc: None,
        }
    }

    pub fn parse(value: &str) -> std::result::Result<Self, String> {
        let mut parts = value.split('|').map(str::trim);
        let name = parts.next().unwrap_or_default().trim();
        if name.is_empty() {
            return Err("mapped struct name is empty".to_string());
        }
        let mut spec = Self::new(rust_type_name(name));
        for option in parts {
            let Some((key, value)) = option.split_once('=') else {
                return Err(format!("mapped option must look like key=value: {option}"));
            };
            match key.trim() {
                "doc" => spec.doc = Some(value.trim().to_string()),
                other => return Err(format!("unknown mapped option: {other}")),
            }
        }
        Ok(spec)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldSpec {
    pub mapped_name: String,
    pub rust_name: String,
    pub key: String,
    pub key_type: FieldKeyType,
    pub field_type: FieldType,
    pub selector: Option<String>,
    pub return_type: Option<String>,
    pub doc: Option<String>,
}

impl FieldSpec {
    pub fn parse(value: &str) -> std::result::Result<Self, String> {
        let mut parts = value.split('|').map(str::trim);
        let head = parts
            .next()
            .ok_or_else(|| "field must look like MappedStruct.field".to_string())?;
        let (mapped_name, rust_name) = head
            .split_once('.')
            .ok_or_else(|| "field must look like MappedStruct.field".to_string())?;
        let mapped_name = rust_type_name(mapped_name.trim());
        let rust_name = rust_fn_name(rust_name.trim());
        if mapped_name.is_empty() || rust_name.is_empty() {
            return Err("field mapping has an empty struct or field name".to_string());
        }
        let mut key = rust_name.clone();
        let mut key_type = FieldKeyType::String;
        let mut field_type = FieldType::String;
        let mut selector = None;
        let mut return_type = None;
        let mut doc = None;
        for option in parts {
            let Some((option_key, value)) = option.split_once('=') else {
                return Err(format!("field option must look like key=value: {option}"));
            };
            match option_key.trim() {
                "key" => key = value.trim().to_string(),
                "key_type" | "keyType" => key_type = FieldKeyType::parse(value.trim())?,
                "type" => field_type = FieldType::parse(value.trim())?,
                "return" => {
                    let raw = value.trim();
                    if let Ok(parsed) = ReturnType::parse(raw) {
                        field_type = FieldType::from_return_type(&parsed);
                        return_type = Some(parsed.config_name().to_string());
                    } else {
                        field_type = FieldType::parse(raw)?;
                        return_type = Some(field_type.config_name());
                    }
                }
                "selector" => selector = Some(parse_field_selector(value.trim())?),
                "doc" => doc = Some(value.trim().to_string()),
                other => return Err(format!("unknown field option: {other}")),
            }
        }
        Ok(Self {
            mapped_name,
            rust_name,
            key,
            key_type,
            field_type,
            selector,
            return_type,
            doc,
        })
    }
}

fn parse_field_selector(value: &str) -> std::result::Result<String, String> {
    if value.is_empty() {
        return Err("field selector is empty".to_string());
    }
    Ok(value.to_string())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FieldKeyType {
    String,
    Symbol,
}

impl FieldKeyType {
    fn parse(value: &str) -> std::result::Result<Self, String> {
        match value {
            "" | "String" | "string" | "str" => Ok(Self::String),
            "Symbol" | "symbol" => Ok(Self::Symbol),
            other => Err(format!("unsupported key_type: {other}")),
        }
    }

    fn config_name(&self) -> &'static str {
        match self {
            Self::String => "String",
            Self::Symbol => "Symbol",
        }
    }

    fn bridge_source(&self) -> &'static str {
        match self {
            Self::String => "BridgeKeyType::String",
            Self::Symbol => "BridgeKeyType::Symbol",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FieldType {
    String,
    SmallInt,
    Bool,
    Oop,
    Mapped(String),
    Vec(Box<FieldType>),
    Map(Box<FieldType>),
    Option(Box<FieldType>),
}

impl FieldType {
    fn parse(value: &str) -> std::result::Result<Self, String> {
        if let Some(inner) = value
            .strip_prefix("Option<")
            .and_then(|text| text.strip_suffix('>'))
        {
            return Ok(Self::Option(Box::new(Self::parse(inner.trim())?)));
        }
        if let Some(inner) = value
            .strip_prefix("Optional<")
            .and_then(|text| text.strip_suffix('>'))
        {
            return Ok(Self::Option(Box::new(Self::parse(inner.trim())?)));
        }
        if let Some(inner) = value
            .strip_prefix("Vec<")
            .and_then(|text| text.strip_suffix('>'))
        {
            return Ok(Self::Vec(Box::new(Self::parse(inner.trim())?)));
        }
        if let Some(inner) = value
            .strip_prefix("Array<")
            .and_then(|text| text.strip_suffix('>'))
        {
            return Ok(Self::Vec(Box::new(Self::parse(inner.trim())?)));
        }
        if let Some(inner) = value
            .strip_prefix("BTreeMap<")
            .and_then(|text| text.strip_suffix('>'))
        {
            return Self::parse_string_keyed_map(inner);
        }
        if let Some(inner) = value
            .strip_prefix("Map<")
            .and_then(|text| text.strip_suffix('>'))
        {
            return Self::parse_string_keyed_map(inner);
        }
        if let Some(inner) = value
            .strip_prefix("Dictionary<")
            .and_then(|text| text.strip_suffix('>'))
        {
            return Ok(Self::Map(Box::new(Self::parse(inner.trim())?)));
        }
        if let Some(inner) = value
            .strip_prefix("Mapped<")
            .and_then(|text| text.strip_suffix('>'))
        {
            return Ok(Self::Mapped(rust_type_name(inner.trim())));
        }
        if let Some(inner) = value
            .strip_prefix("Mapped(")
            .and_then(|text| text.strip_suffix(')'))
        {
            return Ok(Self::Mapped(rust_type_name(inner.trim())));
        }
        match value {
            "" | "String" | "string" => Ok(Self::String),
            "SmallInt" | "smallInt" | "smallint" | "i64" => Ok(Self::SmallInt),
            "Bool" | "Boolean" | "bool" | "boolean" => Ok(Self::Bool),
            "Oop" | "OOP" | "oop" => Ok(Self::Oop),
            other => {
                if other
                    .chars()
                    .next()
                    .is_some_and(|ch| ch.is_ascii_uppercase())
                {
                    Ok(Self::Mapped(rust_type_name(other)))
                } else {
                    Err(format!("unsupported field type: {other}"))
                }
            }
        }
    }

    fn rust_type(&self) -> String {
        match self {
            Self::String => "String".to_string(),
            Self::SmallInt => "i64".to_string(),
            Self::Bool => "bool".to_string(),
            Self::Oop => "Oop".to_string(),
            Self::Mapped(name) => name.clone(),
            Self::Vec(inner) => format!("Vec<{}>", inner.rust_type()),
            Self::Map(inner) => format!("BTreeMap<String, {}>", inner.rust_type()),
            Self::Option(inner) => format!("Option<{}>", inner.rust_type()),
        }
    }

    fn config_name(&self) -> String {
        match self {
            Self::String => "String".to_string(),
            Self::SmallInt => "SmallInt".to_string(),
            Self::Bool => "Bool".to_string(),
            Self::Oop => "Oop".to_string(),
            Self::Mapped(name) => format!("Mapped<{name}>"),
            Self::Vec(inner) => format!("Vec<{}>", inner.config_name()),
            Self::Map(inner) => format!("BTreeMap<String, {}>", inner.config_name()),
            Self::Option(inner) => format!("Option<{}>", inner.config_name()),
        }
    }

    fn parse_string_keyed_map(inner: &str) -> std::result::Result<Self, String> {
        let (key_type, value_type) = inner.split_once(',').ok_or_else(|| {
            "BTreeMap and Map field types must look like BTreeMap<String, T>".to_string()
        })?;
        match key_type.trim() {
            "String" | "string" => Ok(Self::Map(Box::new(Self::parse(value_type.trim())?))),
            other => Err(format!(
                "BTreeMap and Map key type must be String, got {other}"
            )),
        }
    }

    fn uses_btreemap(&self) -> bool {
        match self {
            Self::Map(_) => true,
            Self::Vec(inner) | Self::Option(inner) => inner.uses_btreemap(),
            Self::String | Self::SmallInt | Self::Bool | Self::Oop | Self::Mapped(_) => false,
        }
    }

    fn from_return_type(return_type: &ReturnType) -> Self {
        match return_type {
            ReturnType::String | ReturnType::Symbol => Self::String,
            ReturnType::SmallInt => Self::SmallInt,
            ReturnType::Bool => Self::Bool,
            ReturnType::Oop | ReturnType::Value => Self::Oop,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MethodSpec {
    pub class_ref: ClassRef,
    pub selector: String,
    pub args: Vec<String>,
    pub arg_types: Vec<ArgType>,
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
        let mut arg_types = Vec::new();
        let mut return_type = ReturnType::Value;
        let mut doc = None;
        for option in parts {
            let Some((key, value)) = option.split_once('=') else {
                return Err(format!("method option must look like key=value: {option}"));
            };
            match key.trim() {
                "args" => {
                    let parsed = parse_method_args(value.trim())?;
                    args = parsed.iter().map(|arg| arg.name.clone()).collect();
                    arg_types = parsed.into_iter().map(|arg| arg.arg_type).collect();
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
            arg_types,
            return_type,
            doc,
        })
    }

    fn fn_name(&self) -> String {
        rust_fn_name(&self.selector)
    }

    fn arg_names(&self) -> Vec<String> {
        if self.args.is_empty() {
            inferred_selector_args(&self.selector)
        } else {
            self.args.iter().map(|arg| rust_fn_name(arg)).collect()
        }
    }

    fn arg_specs(&self) -> Vec<MethodArgSpec> {
        let names = self.arg_names();
        names
            .into_iter()
            .enumerate()
            .map(|(index, name)| MethodArgSpec {
                name,
                arg_type: self.arg_types.get(index).cloned().unwrap_or(ArgType::Oop),
            })
            .collect()
    }

    fn config_args(&self) -> Vec<String> {
        self.args
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let arg_type = self.arg_types.get(index).cloned().unwrap_or(ArgType::Oop);
                if arg_type == ArgType::Oop {
                    name.clone()
                } else {
                    format!("{name}:{}", arg_type.config_name())
                }
            })
            .collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MethodArgSpec {
    name: String,
    arg_type: ArgType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArgType {
    Oop,
    String,
    Symbol,
    SmallInt,
    Bool,
}

impl ArgType {
    fn parse(value: &str) -> std::result::Result<Self, String> {
        match value {
            "" | "Oop" | "OOP" | "oop" => Ok(Self::Oop),
            "String" | "string" | "str" => Ok(Self::String),
            "Symbol" | "symbol" => Ok(Self::Symbol),
            "SmallInt" | "smallInt" | "smallint" | "i64" => Ok(Self::SmallInt),
            "Bool" | "Boolean" | "bool" | "boolean" => Ok(Self::Bool),
            other => Err(format!("unsupported argument type: {other}")),
        }
    }

    fn config_name(&self) -> &'static str {
        match self {
            Self::Oop => "Oop",
            Self::String => "String",
            Self::Symbol => "Symbol",
            Self::SmallInt => "SmallInt",
            Self::Bool => "Bool",
        }
    }

    fn rust_type(&self) -> &'static str {
        match self {
            Self::Oop => "Oop",
            Self::String | Self::Symbol => "impl AsRef<str>",
            Self::SmallInt => "i64",
            Self::Bool => "bool",
        }
    }

    fn conversion_source(&self, name: &str) -> String {
        match self {
            Self::Oop => String::new(),
            Self::String => {
                format!("        let {name} = self.session.new_string({name}.as_ref())?;\n")
            }
            Self::Symbol => {
                format!("        let {name} = self.session.new_symbol({name}.as_ref())?;\n")
            }
            Self::SmallInt => format!("        let {name} = self.session.smallint_oop({name});\n"),
            Self::Bool => format!("        let {name} = self.session.bool_oop({name});\n"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReturnType {
    Value,
    String,
    Symbol,
    SmallInt,
    Bool,
    Oop,
}

impl ReturnType {
    fn parse(value: &str) -> std::result::Result<Self, String> {
        match value {
            "" | "Value" | "value" => Ok(Self::Value),
            "String" | "string" => Ok(Self::String),
            "Symbol" | "symbol" => Ok(Self::Symbol),
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
            Self::Symbol => "Symbol",
            Self::SmallInt => "SmallInt",
            Self::Bool => "Bool",
            Self::Oop => "Oop",
        }
    }

    fn rust_type(&self) -> &'static str {
        match self {
            Self::Value => "Value",
            Self::String | Self::Symbol => "String",
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

pub fn explain(config: &Config) -> String {
    let mut out = String::new();
    out.push_str("gemstone-rs codegen explain\n");
    out.push_str(&format!("output: {}\n", config.output.display()));
    out.push_str(&format!(
        "test_stubs: {}\n",
        generated_test_stubs().join(", ")
    ));
    out.push_str(&format!("classes: {}\n", config.classes.len()));
    for class in &config.classes {
        out.push_str(&format!(
            "  class: {} methods={}\n",
            class.class_ref.display_name(),
            class.methods.len()
        ));
        for method in &class.methods {
            let args = method
                .arg_specs()
                .into_iter()
                .map(|arg| format!("{}:{}", arg.name, arg.arg_type.config_name()))
                .collect::<Vec<_>>();
            out.push_str(&format!(
                "    method: {}>>{} args=[{}] return={}\n",
                method.class_ref.display_name(),
                method.selector,
                args.join(", "),
                method.return_type.config_name()
            ));
        }
    }
    out.push_str(&format!("mapped: {}\n", config.mapped.len()));
    for mapped in &config.mapped {
        out.push_str(&format!(
            "  mapped: {} fields={}\n",
            mapped.name,
            mapped.fields.len()
        ));
        for field in &mapped.fields {
            out.push_str(&format!(
                "    field: {}.{} key={} key_type={} type={}",
                field.mapped_name,
                field.rust_name,
                field.key,
                field.key_type.config_name(),
                field.field_type.config_name(),
            ));
            if let Some(selector) = &field.selector {
                out.push_str(&format!(" selector={selector}"));
            }
            if let Some(return_type) = &field.return_type {
                out.push_str(&format!(" return={return_type}"));
            }
            out.push('\n');
        }
    }
    out
}

pub fn explain_json(config: &Config) -> String {
    let classes = config
        .classes
        .iter()
        .map(|class| {
            let methods = class
                .methods
                .iter()
                .map(|method| {
                    let args = method.arg_specs();
                    let arg_names = args.iter().map(|arg| arg.name.clone()).collect::<Vec<_>>();
                    let arg_types = args
                        .iter()
                        .map(|arg| arg.arg_type.config_name().to_string())
                        .collect::<Vec<_>>();
                    let arguments = args
                        .iter()
                        .map(|arg| {
                            format!(
                                r#"{{"name":"{}","type":"{}","rustType":"{}"}}"#,
                                json_escape(&arg.name),
                                arg.arg_type.config_name(),
                                json_escape(arg.arg_type.rust_type())
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(",");
                    format!(
                        r#"{{"selector":"{}","args":[{}],"argTypes":[{}],"arguments":[{}],"return":"{}","doc":{}}}"#,
                        json_escape(&method.selector),
                        json_string_array(&arg_names),
                        json_string_array(&arg_types),
                        arguments,
                        method.return_type.config_name(),
                        optional_json_string(method.doc.as_deref())
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(
                r#"{{"name":"{}","dictionary":"{}","className":"{}","meta":{},"methods":[{}]}}"#,
                json_escape(&class.class_ref.display_name()),
                json_escape(&class.class_ref.dictionary),
                json_escape(&class.class_ref.class_name),
                class.class_ref.meta,
                methods
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let mapped = config
        .mapped
        .iter()
        .map(|mapped| {
            let fields = mapped
                .fields
                .iter()
                .map(|field| {
                    format!(
                        r#"{{"name":"{}","key":"{}","keyType":"{}","type":"{}","selector":{},"return":{},"doc":{}}}"#,
                        json_escape(&field.rust_name),
                        json_escape(&field.key),
                        field.key_type.config_name(),
                        field.field_type.config_name(),
                        optional_json_string(field.selector.as_deref()),
                        optional_json_string(field.return_type.as_deref()),
                        optional_json_string(field.doc.as_deref())
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(
                r#"{{"name":"{}","doc":{},"fields":[{}]}}"#,
                json_escape(&mapped.name),
                optional_json_string(mapped.doc.as_deref()),
                fields
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let test_stubs = generated_test_stubs()
        .iter()
        .map(|stub| (*stub).to_string())
        .collect::<Vec<_>>();
    format!(
        r#"{{"output":"{}","testStubs":[{}],"classes":[{}],"mapped":[{}]}}"#,
        json_escape(&config.output.display().to_string()),
        json_string_array(&test_stubs),
        classes,
        mapped
    )
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
        let selector_protocols = discover_selector_protocols(&mut browser, &class_ref);
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
            let protocol = selector_protocols.get(&selector).map(String::as_str);
            spec.methods.push(discovered_method_spec(
                &class_ref, selector, protocol, &source,
            ));
        }
        specs.push(spec);
    }
    Ok(Config {
        output,
        classes: specs,
        mapped: Vec::new(),
    })
}

fn discover_selector_protocols(
    browser: &mut Browser<'_>,
    class_ref: &ClassRef,
) -> BTreeMap<String, String> {
    let mut selector_protocols = BTreeMap::new();
    let protocols = browser
        .protocols(&class_ref.class_name, class_ref.meta, &class_ref.dictionary)
        .unwrap_or_default();
    for protocol in protocols {
        for selector in browser
            .methods(
                &class_ref.class_name,
                &protocol,
                class_ref.meta,
                &class_ref.dictionary,
            )
            .unwrap_or_default()
        {
            selector_protocols
                .entry(selector)
                .or_insert_with(|| protocol.clone());
        }
    }
    selector_protocols
}

fn discovered_method_spec(
    class_ref: &ClassRef,
    selector: String,
    protocol: Option<&str>,
    source: &str,
) -> MethodSpec {
    let args = source_header_arg_names(&selector, source)
        .unwrap_or_else(|| inferred_selector_args(&selector));
    let arg_types = vec![ArgType::Oop; args.len()];
    MethodSpec {
        class_ref: class_ref.clone(),
        selector,
        args,
        arg_types,
        return_type: ReturnType::Value,
        doc: discovery_doc(protocol, source),
    }
}

fn discovery_doc(protocol: Option<&str>, source: &str) -> Option<String> {
    let protocol = protocol.map(str::trim).filter(|value| !value.is_empty());
    let source_line = first_source_line(source);
    match (protocol, source_line) {
        (Some(protocol), Some(source_line)) => Some(format!("protocol {protocol}; {source_line}")),
        (Some(protocol), None) => Some(format!("protocol {protocol}")),
        (None, Some(source_line)) => Some(source_line),
        (None, None) => None,
    }
}

pub fn discover_mapping(
    session: &mut Session,
    output: PathBuf,
    mapped_name: &str,
    class_ref: &ClassRef,
) -> Result<Config> {
    let class_oop = session.execute(&browser::behavior_expr(
        &class_ref.class_name,
        class_ref.meta,
        &class_ref.dictionary,
    ))?;
    let names_oop = session.perform_oop(class_oop, "allInstVarNames", &[])?;
    let mut fields = Vec::new();
    for name in session.array_strings(names_oop)? {
        let rust_name = rust_fn_name(&name);
        fields.push(FieldSpec {
            mapped_name: rust_type_name(mapped_name),
            rust_name,
            key: name,
            key_type: FieldKeyType::Symbol,
            field_type: FieldType::String,
            selector: None,
            return_type: None,
            doc: Some("Discovered from GemStone instance variable name.".to_string()),
        });
    }
    if fields.is_empty() {
        fields.push(FieldSpec {
            mapped_name: rust_type_name(mapped_name),
            rust_name: "name".to_string(),
            key: "name".to_string(),
            key_type: FieldKeyType::String,
            field_type: FieldType::String,
            selector: None,
            return_type: None,
            doc: Some("Placeholder field; edit after discovery.".to_string()),
        });
    }
    Ok(Config {
        output,
        classes: Vec::new(),
        mapped: vec![MappedSpec {
            name: rust_type_name(mapped_name),
            fields,
            doc: Some(format!(
                "Mapping proposal discovered from {}.",
                class_ref.display_name()
            )),
        }],
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
                source.push_str(&method.config_args().join(","));
            }
            if method.return_type != ReturnType::Value {
                source.push_str(" | return=");
                source.push_str(method.return_type.config_name());
            }
            if let Some(doc) = method.doc.as_deref().filter(|doc| !doc.is_empty()) {
                source.push_str(" | doc=");
                source.push_str(&config_doc(doc));
            }
            source.push('\n');
        }
    }
    for mapped in &config.mapped {
        source.push_str(&format!("mapped = {}", mapped.name));
        if let Some(doc) = mapped.doc.as_deref().filter(|doc| !doc.is_empty()) {
            source.push_str(" | doc=");
            source.push_str(&doc.replace('\n', " "));
        }
        source.push('\n');
        for field in &mapped.fields {
            source.push_str(&format!(
                "field = {}.{} | type={} | key={}",
                mapped.name,
                field.rust_name,
                field.field_type.config_name(),
                field.key
            ));
            if field.key_type != FieldKeyType::String {
                source.push_str(" | key_type=");
                source.push_str(field.key_type.config_name());
            }
            if let Some(selector) = &field.selector {
                source.push_str(" | selector=");
                source.push_str(selector);
            }
            if let Some(return_type) = &field.return_type {
                source.push_str(" | return=");
                source.push_str(return_type);
            }
            if let Some(doc) = field.doc.as_deref().filter(|doc| !doc.is_empty()) {
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
     method = Object>>class\n\
     method = Object>>perform: | args=selector:Symbol | doc=Perform a unary selector supplied as a Rust string.\n\
     mapped = BookingDraft | doc=A typed Rust payload stored under BridgeRoot.\n\
     field = BookingDraft.name | type=String | key=name\n\
     field = BookingDraft.amount | type=SmallInt | key=amount\n\
     field = BookingDraft.currency | type=String | key=currency\n\
     field = BookingDraft.tags | type=Vec<String> | key=tags\n\
     field = BookingDraft.labels | type=BTreeMap<String, String> | key=labels\n\
     field = BookingDraft.note | type=Option<String> | key=note\n"
}

pub fn sample_mapping_config(mapped: &str) -> String {
    let mapped = rust_type_name(mapped);
    format!(
        "mapped = {mapped} | doc=Typed payload stored under GemStoneRsBridgeRoot.\n\
         field = {mapped}.name | type=String | key=name | key_type=String\n\
         field = {mapped}.amount | type=SmallInt | key=amount | key_type=String\n\
         field = {mapped}.tags | type=Vec<String> | key=tags | key_type=String\n\
         field = {mapped}.labels | type=BTreeMap<String, String> | key=labels | key_type=String\n\
         field = {mapped}.note | type=Option<String> | key=note | key_type=String\n"
    )
}

pub fn mapping_config_from_bridge_value(mapped: &str, value: &BridgeValue) -> String {
    let mut inference = BridgeMappingInference::new(rust_type_name(mapped));
    let root_name = inference.root_name.clone();
    inference.infer_mapping(&root_name, "Inferred from a live BridgeRoot value.", value);
    inference.render()
}

#[derive(Debug)]
struct BridgeMappingInference {
    root_name: String,
    mappings: Vec<InferredMapping>,
}

impl BridgeMappingInference {
    fn new(root_name: String) -> Self {
        Self {
            root_name,
            mappings: Vec::new(),
        }
    }

    fn infer_mapping(&mut self, name: &str, doc: &str, value: &BridgeValue) {
        if self.mappings.iter().any(|mapping| mapping.name == name) {
            return;
        }

        self.mappings.push(InferredMapping {
            name: name.to_string(),
            doc: doc.to_string(),
            fields: Vec::new(),
        });

        let fields = match value {
            BridgeValue::Dictionary(entries) => {
                let mut fields = Vec::new();
                for (key, value) in entries {
                    fields.push(self.infer_field(name, key, BridgeKeyType::String, value));
                }
                fields
            }
            BridgeValue::KeyedDictionary(entries) => {
                let mut fields = Vec::new();
                for (key, value) in entries {
                    fields.push(self.infer_field(name, &key.name, key.key_type, value));
                }
                fields
            }
            other => {
                let (field_type, doc) = self.infer_field_type(name, "value", other);
                vec![InferredField {
                    rust_name: "value".to_string(),
                    key: "value".to_string(),
                    key_type: BridgeKeyType::String,
                    field_type,
                    doc: Some(doc.unwrap_or_else(|| {
                        "Synthetic field for a scalar BridgeRoot value; review the key before generating wrappers.".to_string()
                    })),
                }]
            }
        };

        if let Some(mapping) = self
            .mappings
            .iter_mut()
            .find(|mapping| mapping.name == name)
        {
            mapping.fields = fields;
        }
    }

    fn infer_field(
        &mut self,
        parent: &str,
        key: &str,
        key_type: BridgeKeyType,
        value: &BridgeValue,
    ) -> InferredField {
        let rust_name = rust_fn_name(key);
        let (field_type, doc) = self.infer_field_type(parent, key, value);
        InferredField {
            rust_name,
            key: key.to_string(),
            key_type,
            field_type,
            doc,
        }
    }

    fn infer_field_type(
        &mut self,
        parent: &str,
        key: &str,
        value: &BridgeValue,
    ) -> (FieldType, Option<String>) {
        match value {
            BridgeValue::Nil => (
                FieldType::Option(Box::new(FieldType::Oop)),
                Some("Observed nil; choose a narrower Option<T> before committing generated code.".to_string()),
            ),
            BridgeValue::Bool(_) => (FieldType::Bool, None),
            BridgeValue::SmallInt(_) => (FieldType::SmallInt, None),
            BridgeValue::String(_) => (FieldType::String, None),
            BridgeValue::Symbol(_) => (
                FieldType::String,
                Some("Observed a GemStone Symbol; generated as String because BridgeMapped fields currently store string-like values explicitly.".to_string()),
            ),
            BridgeValue::Oop(oop) => (
                FieldType::Oop,
                Some(format!(
                    "Observed opaque OOP {}; inspect it or increase --depth before choosing a mapped type.",
                    oop.raw()
                )),
            ),
            BridgeValue::Dictionary(_) | BridgeValue::KeyedDictionary(_) => {
                let nested = nested_mapping_name(parent, key, false);
                self.infer_mapping(&nested, &format!("Nested payload inferred from field `{key}`."), value);
                (FieldType::Mapped(nested), None)
            }
            BridgeValue::Array(values) => self.infer_array_field_type(parent, key, values),
        }
    }

    fn infer_array_field_type(
        &mut self,
        parent: &str,
        key: &str,
        values: &[BridgeValue],
    ) -> (FieldType, Option<String>) {
        let Some(first) = values.first() else {
            return (
                FieldType::Vec(Box::new(FieldType::Oop)),
                Some(
                    "Observed an empty Array; choose the element type before generating wrappers."
                        .to_string(),
                ),
            );
        };

        if values.iter().all(is_dictionary_value) {
            let nested = nested_mapping_name(parent, key, true);
            self.infer_mapping(
                &nested,
                &format!("Array element payload inferred from the first `{key}` entry."),
                first,
            );
            return (FieldType::Vec(Box::new(FieldType::Mapped(nested))), None);
        }

        let (first_type, first_doc) = self.infer_scalar_array_type(first);
        if values
            .iter()
            .all(|value| self.infer_scalar_array_type(value).0 == first_type)
        {
            return (FieldType::Vec(Box::new(first_type)), first_doc);
        }

        (
            FieldType::Vec(Box::new(FieldType::Oop)),
            Some(
                "Observed a mixed Array; inspect values and choose a narrower element type."
                    .to_string(),
            ),
        )
    }

    fn infer_scalar_array_type(&self, value: &BridgeValue) -> (FieldType, Option<String>) {
        match value {
            BridgeValue::Nil => (
                FieldType::Option(Box::new(FieldType::Oop)),
                Some("Observed nil elements; choose a narrower Option<T> element type.".to_string()),
            ),
            BridgeValue::Bool(_) => (FieldType::Bool, None),
            BridgeValue::SmallInt(_) => (FieldType::SmallInt, None),
            BridgeValue::String(_) => (FieldType::String, None),
            BridgeValue::Symbol(_) => (
                FieldType::String,
                Some("Observed Symbol elements; generated as String values.".to_string()),
            ),
            BridgeValue::Oop(_) | BridgeValue::Dictionary(_) | BridgeValue::KeyedDictionary(_) | BridgeValue::Array(_) => (
                FieldType::Oop,
                Some("Observed complex or opaque Array elements; inspect before choosing a mapped element type.".to_string()),
            ),
        }
    }

    fn render(&self) -> String {
        let mut source = String::new();
        source.push_str(
            "# Generated by gemstone-rs from a live BridgeValue. Review before using in CI.\n",
        );
        source.push_str("output = src/generated/gemstone_wrappers.rs\n");
        for mapping in &self.mappings {
            source.push_str(&format!(
                "mapped = {} | doc={}\n",
                mapping.name,
                config_doc(&mapping.doc)
            ));
            for field in &mapping.fields {
                source.push_str(&format!(
                    "field = {}.{} | type={} | key={} | key_type={}",
                    mapping.name,
                    field.rust_name,
                    field.field_type.config_name(),
                    field.key,
                    field.key_type.config_name()
                ));
                if let Some(doc) = &field.doc {
                    source.push_str(&format!(" | doc={}", config_doc(doc)));
                }
                source.push('\n');
            }
        }
        source
    }
}

#[derive(Debug)]
struct InferredMapping {
    name: String,
    doc: String,
    fields: Vec<InferredField>,
}

#[derive(Debug)]
struct InferredField {
    rust_name: String,
    key: String,
    key_type: BridgeKeyType,
    field_type: FieldType,
    doc: Option<String>,
}

fn is_dictionary_value(value: &BridgeValue) -> bool {
    matches!(
        value,
        BridgeValue::Dictionary(_) | BridgeValue::KeyedDictionary(_)
    )
}

fn nested_mapping_name(parent: &str, key: &str, array_element: bool) -> String {
    let key = if array_element {
        singular_field_name(key)
    } else {
        key.to_string()
    };
    rust_type_name(&format!("{parent} {key}"))
}

fn singular_field_name(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.len() > 3 && trimmed.ends_with("ies") {
        format!("{}y", &trimmed[..trimmed.len() - 3])
    } else if trimmed.len() > 1 && trimmed.ends_with('s') {
        trimmed[..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

fn config_doc(value: &str) -> String {
    value.replace(['\r', '\n', '|'], " ")
}

fn generate_source(config: &Config) -> String {
    let mut source = String::new();
    source.push_str("// @generated by gemstone-rs codegen. Do not edit by hand.\n");
    source.push_str(
        "use gemstone_rs::{\n    BridgeDictionary, BridgeFieldRead, BridgeFieldWrite, BridgeKey, BridgeKeyType, BridgeMapped,\n    BridgeValue, Error, Oop, Result, Session, Value,\n};\n\n",
    );
    if config_uses_btreemap(config) {
        source.push_str("use std::collections::BTreeMap;\n\n");
    }

    for class in &config.classes {
        let struct_name = class.class_ref.struct_name();
        source.push_str(&format!("pub struct {struct_name}<'a> {{\n"));
        source.push_str("    session: &'a mut Session,\n");
        source.push_str("    oop: Oop,\n");
        source.push_str("}\n\n");
        source.push_str(&format!("impl<'a> {struct_name}<'a> {{\n"));
        source.push_str("    pub fn resolve(session: &'a mut Session) -> Result<Self> {\n");
        source.push_str("        let oop =\n");
        source.push_str(&format!(
            "            session.execute({})?;\n",
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

    for mapped in &config.mapped {
        source.push_str(&mapped_source(mapped));
        source.push('\n');
    }
    source.push_str(&test_stubs_source(config));

    while source.ends_with("\n\n") {
        source.pop();
    }
    source
}

fn config_uses_btreemap(config: &Config) -> bool {
    config.mapped.iter().any(|mapped| {
        mapped
            .fields
            .iter()
            .any(|field| field.field_type.uses_btreemap())
    })
}

fn generated_test_stubs() -> &'static [&'static str] {
    &[
        "generated_surface_names_are_stable",
        "generated_method_metadata_is_stable",
        "generated_mapped_field_metadata_is_stable",
    ]
}

fn test_stubs_source(config: &Config) -> String {
    let mut names = Vec::new();
    let mut method_metadata = Vec::new();
    let mut field_metadata = Vec::new();
    for class in &config.classes {
        let struct_name = class.class_ref.struct_name();
        if class.methods.is_empty() {
            names.push(struct_name);
        } else {
            for method in &class.methods {
                let fn_name = method.fn_name();
                let args = method
                    .arg_specs()
                    .into_iter()
                    .map(|arg| format!("{}:{}", arg.name, arg.arg_type.config_name()))
                    .collect::<Vec<_>>()
                    .join(",");
                names.push(format!("{struct_name}::{fn_name}"));
                method_metadata.push((
                    struct_name.clone(),
                    fn_name,
                    method.selector.clone(),
                    args,
                    method.return_type.config_name().to_string(),
                ));
            }
        }
    }
    for mapped in &config.mapped {
        names.push(mapped.name.clone());
        for field in &mapped.fields {
            field_metadata.push((
                mapped.name.clone(),
                field.rust_name.clone(),
                field.key.clone(),
                field.key_type.config_name().to_string(),
                field.field_type.config_name(),
            ));
        }
    }

    let mut source = String::new();
    source.push_str("#[cfg(test)]\n");
    source.push_str("#[rustfmt::skip]\n");
    source.push_str("mod generated_code_tests {\n");
    source.push_str("    #[test]\n");
    source.push_str("    fn generated_surface_names_are_stable() {\n");
    source.push_str("        let names: &[&str] = &[\n");
    for name in names {
        source.push_str("            ");
        source.push_str(&rust_string_literal(&name));
        source.push_str(",\n");
    }
    source.push_str("        ];\n");
    source.push_str("        assert!(names.iter().all(|name| !name.is_empty()));\n");
    source.push_str("    }\n");
    source.push('\n');
    source.push_str("    #[test]\n");
    source.push_str("    fn generated_method_metadata_is_stable() {\n");
    source.push_str("        let methods: &[(&str, &str, &str, &str, &str)] = &[\n");
    for (struct_name, fn_name, selector, args, return_type) in method_metadata {
        source.push_str("            (");
        source.push_str(&rust_string_literal(&struct_name));
        source.push_str(", ");
        source.push_str(&rust_string_literal(&fn_name));
        source.push_str(", ");
        source.push_str(&rust_string_literal(&selector));
        source.push_str(", ");
        source.push_str(&rust_string_literal(&args));
        source.push_str(", ");
        source.push_str(&rust_string_literal(&return_type));
        source.push_str("),\n");
    }
    source.push_str("        ];\n");
    source.push_str("        assert!(methods.iter().all(|(_, fn_name, selector, _, return_type)| !fn_name.is_empty() && !selector.is_empty() && !return_type.is_empty()));\n");
    source.push_str("        assert!(methods.iter().all(|(_, _, selector, args, _)| selector.matches(':').count() == if args.is_empty() { 0 } else { args.split(',').count() }));\n");
    source.push_str("    }\n");
    source.push('\n');
    source.push_str("    #[test]\n");
    source.push_str("    fn generated_mapped_field_metadata_is_stable() {\n");
    source.push_str("        let fields: &[(&str, &str, &str, &str, &str)] = &[\n");
    for (mapped_name, rust_name, key, key_type, field_type) in field_metadata {
        source.push_str("            (");
        source.push_str(&rust_string_literal(&mapped_name));
        source.push_str(", ");
        source.push_str(&rust_string_literal(&rust_name));
        source.push_str(", ");
        source.push_str(&rust_string_literal(&key));
        source.push_str(", ");
        source.push_str(&rust_string_literal(&key_type));
        source.push_str(", ");
        source.push_str(&rust_string_literal(&field_type));
        source.push_str("),\n");
    }
    source.push_str("        ];\n");
    source.push_str("        assert!(fields.iter().all(|(mapped_name, rust_name, key, _, field_type)| !mapped_name.is_empty() && !rust_name.is_empty() && !key.is_empty() && !field_type.is_empty()));\n");
    source.push_str("        assert!(fields.iter().all(|(_, _, _, key_type, _)| matches!(*key_type, \"String\" | \"Symbol\")));\n");
    source.push_str("    }\n");
    source.push_str("}\n");
    source
}

fn mapped_source(mapped: &MappedSpec) -> String {
    let mut source = String::new();
    if let Some(doc) = mapped.doc.as_deref().filter(|doc| !doc.is_empty()) {
        source.push_str(&format!("/// {}\n", escape_doc(doc)));
    }
    source.push_str("#[derive(Clone, Debug, Eq, PartialEq)]\n");
    source.push_str(&format!("pub struct {} {{\n", mapped.name));
    for field in &mapped.fields {
        if let Some(doc) = field.doc.as_deref().filter(|doc| !doc.is_empty()) {
            source.push_str(&format!("    /// {}\n", escape_doc(doc)));
        }
        source.push_str(&format!(
            "    pub {}: {},\n",
            field.rust_name,
            field.field_type.rust_type()
        ));
    }
    source.push_str("}\n\n");
    source.push_str(&format!("impl BridgeMapped for {} {{\n", mapped.name));
    source.push_str("    fn to_bridge_value(&self) -> BridgeValue {\n");
    source.push_str("        BridgeValue::keyed_dictionary([\n");
    for field in &mapped.fields {
        source.push_str(&mapped_field_write(field));
    }
    source.push_str("        ])\n");
    source.push_str("    }\n\n");
    source.push_str(
        "    fn from_bridge_dictionary(dictionary: &mut BridgeDictionary<'_>) -> Result<Self> {\n",
    );
    source.push_str("        Ok(Self {\n");
    for field in &mapped.fields {
        source.push_str(&mapped_field_read(field));
    }
    source.push_str("        })\n");
    source.push_str("    }\n");
    source.push_str("}\n");
    source
}

fn mapped_field_write(field: &FieldSpec) -> String {
    format!(
        "            (\n                BridgeKey::new({}, {}),\n                BridgeFieldWrite::to_bridge_field_value(&self.{}),\n            ),\n",
        rust_string_literal(&field.key),
        field.key_type.bridge_source(),
        field.rust_name
    )
}

fn mapped_field_read(field: &FieldSpec) -> String {
    let inline = format!(
        "            {}: BridgeFieldRead::read_bridge_field(dictionary, {}, {})?,\n",
        field.rust_name,
        rust_string_literal(&field.key),
        field.key_type.bridge_source()
    );
    if inline.trim_end().len() <= 100 {
        return inline;
    }
    format!(
        "            {}: BridgeFieldRead::read_bridge_field(\n                dictionary,\n                {},\n                {},\n            )?,\n",
        field.rust_name,
        rust_string_literal(&field.key),
        field.key_type.bridge_source()
    )
}

fn method_source(method: &MethodSpec) -> String {
    let mut source = String::new();
    let fn_name = method.fn_name();
    let arg_specs = method.arg_specs();
    let args: Vec<String> = arg_specs
        .iter()
        .map(|arg| format!("{}: {}", arg.name, arg.arg_type.rust_type()))
        .collect();
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
    for arg in &arg_specs {
        source.push_str(&arg.arg_type.conversion_source(&arg.name));
    }
    let arg_names = arg_specs
        .iter()
        .map(|arg| arg.name.clone())
        .collect::<Vec<_>>();
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
        ReturnType::Symbol => typed_match(
            "Symbol",
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

fn parse_method_args(value: &str) -> std::result::Result<Vec<MethodArgSpec>, String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|arg| !arg.is_empty())
        .map(|arg| {
            let (name, arg_type) = arg
                .split_once(':')
                .map(|(name, arg_type)| (name.trim(), ArgType::parse(arg_type.trim())))
                .unwrap_or((arg, Ok(ArgType::Oop)));
            let name = name.trim();
            if name.is_empty() {
                return Err("method argument name is empty".to_string());
            }
            Ok(MethodArgSpec {
                name: name.to_string(),
                arg_type: arg_type?,
            })
        })
        .collect()
}

fn first_source_line(source: &str) -> Option<String> {
    source
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(|line| line.chars().take(120).collect())
}

fn source_header_arg_names(selector: &str, source: &str) -> Option<Vec<String>> {
    let keywords = selector_keywords(selector);
    if keywords.is_empty() {
        return Some(Vec::new());
    }
    let header = first_source_line(source)?;
    let tokens = header.split_whitespace().collect::<Vec<_>>();
    let mut cursor = 0;
    let mut args = Vec::new();
    for keyword in keywords {
        let index = tokens
            .iter()
            .enumerate()
            .skip(cursor)
            .find_map(|(index, token)| (clean_header_token(token) == keyword).then_some(index))?;
        let raw_arg = tokens.get(index + 1)?;
        if !looks_like_smalltalk_argument(raw_arg) {
            return None;
        }
        let name = rust_fn_name(&clean_header_token(raw_arg));
        if name.is_empty() || name == "perform" {
            return None;
        }
        args.push(name);
        cursor = index + 2;
    }
    Some(args)
}

fn selector_keywords(selector: &str) -> Vec<String> {
    if !selector.contains(':') {
        return Vec::new();
    }
    selector
        .split(':')
        .take_while(|part| !part.is_empty())
        .map(|part| format!("{part}:"))
        .collect()
}

fn clean_header_token(token: &str) -> String {
    token
        .trim_matches(|ch: char| matches!(ch, '(' | ')' | '[' | ']' | '{' | '}' | '.' | ',' | ';'))
        .to_string()
}

fn looks_like_smalltalk_argument(token: &str) -> bool {
    let token = clean_header_token(token);
    let mut chars = token.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn inferred_selector_args(selector: &str) -> Vec<String> {
    if !selector.contains(':') {
        return Vec::new();
    }
    let keywords: Vec<&str> = selector
        .split(':')
        .take_while(|part| !part.is_empty())
        .collect();
    if keywords.is_empty() {
        return Vec::new();
    }
    keywords
        .iter()
        .enumerate()
        .map(|(index, keyword)| {
            let candidate = keyword
                .strip_prefix("with")
                .filter(|rest| !rest.is_empty())
                .unwrap_or(keyword);
            let name = rust_fn_name(candidate);
            if name.is_empty() || name == "perform" {
                format!("arg{index}")
            } else {
                name
            }
        })
        .collect()
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

fn json_escape(value: &str) -> String {
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

fn json_string_array(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!(r#""{}""#, json_escape(value)))
        .collect::<Vec<_>>()
        .join(",")
}

fn optional_json_string(value: Option<&str>) -> String {
    value
        .map(|value| format!(r#""{}""#, json_escape(value)))
        .unwrap_or_else(|| "null".to_string())
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
        assert_eq!(config.classes[1].methods[0].arg_types, vec![ArgType::Oop]);
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
    fn infers_method_argument_names_from_selector_keywords() -> Result<()> {
        let method = MethodSpec::parse("Order>>findById:ifAbsent:").unwrap();
        assert_eq!(method.arg_names(), vec!["find_by_id", "if_absent"]);

        let generated = generate(&Config::parse(
            "class = Object\nmethod = Object>>at:put:\nmethod = Object>>withCustomer:amount:\n",
            None,
        )?);
        assert!(generated
            .source
            .contains("pub fn at_put(&mut self, at: Oop, put: Oop)"));
        assert!(generated
            .source
            .contains("pub fn with_customer_amount(&mut self, customer: Oop, amount: Oop)"));
        Ok(())
    }

    #[test]
    fn extracts_argument_names_from_smalltalk_source_headers() {
        assert_eq!(
            source_header_arg_names(
                "withCustomer:amount:",
                "withCustomer: aCustomer amount: totalAmount\n  ^self"
            ),
            Some(vec!["a_customer".to_string(), "total_amount".to_string()])
        );
        assert_eq!(
            source_header_arg_names(
                "_changeClassTo:preserveVarying:",
                "_changeClassTo: aClass preserveVarying: aBoolean"
            ),
            Some(vec!["a_class".to_string(), "a_boolean".to_string()])
        );
        assert_eq!(
            source_header_arg_names("printString", "printString"),
            Some(Vec::new())
        );
        assert_eq!(source_header_arg_names("at:put:", "at: ^self"), None);
    }

    #[test]
    fn discovered_methods_include_safe_argument_and_protocol_metadata() -> Result<()> {
        let class_ref = ClassRef::parse("Object").unwrap();
        let method = discovered_method_spec(
            &class_ref,
            "at:put:".to_string(),
            Some("accessing"),
            "at: key put: value\n  ^self",
        );

        assert_eq!(method.args, vec!["key", "value"]);
        assert_eq!(method.arg_types, vec![ArgType::Oop, ArgType::Oop]);
        assert_eq!(method.return_type, ReturnType::Value);
        assert_eq!(
            method.doc.as_deref(),
            Some("protocol accessing; at: key put: value")
        );

        let config = Config {
            output: PathBuf::from("generated.rs"),
            classes: vec![ClassSpec {
                class_ref,
                methods: vec![method],
            }],
            mapped: Vec::new(),
        };
        let source = config_source(&config);
        assert!(source.contains(
            "method = Object>>at:put: | args=key,value | doc=protocol accessing; at: key put: value"
        ));
        let json = explain_json(&config);
        assert!(json.contains(r#""args":["key","value"]"#));
        assert!(json.contains(r#""argTypes":["Oop","Oop"]"#));
        Ok(())
    }

    #[test]
    fn parses_and_generates_typed_method_arguments() -> Result<()> {
        let config = Config::parse(
            "class = UserGlobals:Order\nmethod = UserGlobals:Order>>findById:customer:active: | args=id:SmallInt,customer:String,active:Bool | return=Oop\nmethod = UserGlobals:Order>>findBySymbol: | args=key:Symbol | return=Oop\n",
            None,
        )?;
        let method = &config.classes[0].methods[0];
        assert_eq!(
            method.arg_types,
            vec![ArgType::SmallInt, ArgType::String, ArgType::Bool]
        );
        assert_eq!(
            method.config_args(),
            vec!["id:SmallInt", "customer:String", "active:Bool"]
        );

        let generated = generate(&config);
        assert!(generated.source.contains(
            "pub fn find_by_id_customer_active(&mut self, id: i64, customer: impl AsRef<str>, active: bool) -> Result<Oop>"
        ));
        assert!(generated
            .source
            .contains("let id = self.session.smallint_oop(id);"));
        assert!(generated
            .source
            .contains("let customer = self.session.new_string(customer.as_ref())?;"));
        assert!(generated
            .source
            .contains("let active = self.session.bool_oop(active);"));
        assert!(generated
            .source
            .contains("pub fn find_by_symbol(&mut self, key: impl AsRef<str>) -> Result<Oop>"));
        assert!(generated
            .source
            .contains("let key = self.session.new_symbol(key.as_ref())?;"));

        let explanation = explain(&config);
        assert!(explanation.contains("args=[id:SmallInt, customer:String, active:Bool]"));
        let json = explain_json(&config);
        assert!(json.contains(r#""argTypes":["SmallInt","String","Bool"]"#));
        assert!(json.contains(r#""arguments":[{"name":"id","type":"SmallInt","rustType":"i64"}"#));
        Ok(())
    }

    #[test]
    fn parses_and_generates_symbol_return_helpers() -> Result<()> {
        let config = Config::parse(
            "class = UserGlobals:Order\nmethod = UserGlobals:Order>>statusSymbol | return=Symbol\n",
            None,
        )?;
        assert_eq!(config.classes[0].methods[0].return_type, ReturnType::Symbol);

        let generated = generate(&config);
        assert!(generated
            .source
            .contains("pub fn status_symbol(&mut self) -> Result<String>"));
        assert!(generated.source.contains("expected: \"Symbol\""));
        assert!(generated
            .source
            .contains("Value::Oop(oop) => self.session.fetch_string(oop)"));

        let explanation = explain(&config);
        assert!(
            explanation.contains("method: UserGlobals:Order>>statusSymbol args=[] return=Symbol")
        );
        let json = explain_json(&config);
        assert!(json.contains(r#""return":"Symbol""#));
        Ok(())
    }

    #[test]
    fn generates_wrapper_source() -> Result<()> {
        let config = Config::parse(
            "class = Object\nmethod = Object>>printString | return=String | doc=Print the receiver.\nmethod = Object>>at:put: | args=key,value\n",
            None,
        )?;
        let generated = generate(&config);
        assert!(generated.source.contains("pub struct Object<'a>"));
        assert!(generated.source.contains("mod generated_code_tests"));
        assert!(generated
            .source
            .contains("fn generated_method_metadata_is_stable()"));
        assert!(generated
            .source
            .contains("(\"Object\", \"print_string\", \"printString\", \"\", \"String\")"));
        assert!(generated
            .source
            .contains("(\"Object\", \"at_put\", \"at:put:\", \"key:Oop,value:Oop\", \"Value\")"));
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
    fn explains_codegen_config() -> Result<()> {
        let config = Config::parse(
            "output = generated.rs\nclass = Object\nmethod = Object>>printString | return=String\nmapped = BookingDraft\nfield = BookingDraft.amount | type=SmallInt | key=amount | key_type=Symbol\n",
            None,
        )?;
        let explanation = explain(&config);

        assert!(explanation.contains("output: ./generated.rs"));
        assert!(explanation.contains("test_stubs: generated_surface_names_are_stable"));
        assert!(explanation.contains("generated_method_metadata_is_stable"));
        assert!(explanation.contains("generated_mapped_field_metadata_is_stable"));
        assert!(explanation.contains("class: Object methods=1"));
        assert!(explanation.contains("method: Object>>printString args=[] return=String"));
        assert!(explanation.contains("mapped: BookingDraft fields=1"));
        assert!(explanation
            .contains("field: BookingDraft.amount key=amount key_type=Symbol type=SmallInt"));
        let json = explain_json(&config);
        assert!(json.contains(r#""output":"./generated.rs""#));
        assert!(json.contains(
            r#""testStubs":["generated_surface_names_are_stable","generated_method_metadata_is_stable","generated_mapped_field_metadata_is_stable"]"#
        ));
        assert!(json.contains(r#""selector":"printString""#));
        assert!(json.contains(r#""keyType":"Symbol""#));
        Ok(())
    }

    #[test]
    fn generates_bridge_mapped_struct_source() -> Result<()> {
        let config = Config::parse(
            "mapped = BookingDraft | doc=Payload stored under BridgeRoot.\nfield = BookingDraft.name | type=String | key=name\nfield = BookingDraft.amount | type=SmallInt | key=amount\nfield = BookingDraft.approved | type=Bool | key=approved\n",
            None,
        )?;
        assert_eq!(config.mapped.len(), 1);
        assert_eq!(config.mapped[0].fields.len(), 3);

        let generated = generate(&config);
        assert!(generated.source.contains("pub struct BookingDraft"));
        assert!(generated
            .source
            .contains("impl BridgeMapped for BookingDraft"));
        assert!(generated.source.contains("pub amount: i64"));
        assert!(generated
            .source
            .contains("amount: BridgeFieldRead::read_bridge_field"));
        assert!(generated
            .source
            .contains("BridgeFieldWrite::to_bridge_field_value(&self.approved)"));
        Ok(())
    }

    #[test]
    fn parses_symbol_keys_and_nested_field_types() -> Result<()> {
        let config = Config::parse(
            "mapped = BookingDraft\nfield = BookingDraft.customer | type=Mapped<Customer> | key=customer | key_type=Symbol\nfield = BookingDraft.tags | type=Vec<String> | key=tags\nfield = BookingDraft.labels | type=BTreeMap<String, String> | key=labels\nfield = BookingDraft.note | type=Option<String> | key=note\n",
            None,
        )?;
        let fields = &config.mapped[0].fields;
        assert_eq!(fields[0].key_type, FieldKeyType::Symbol);
        assert_eq!(
            fields[0].field_type,
            FieldType::Mapped("Customer".to_string())
        );
        assert_eq!(
            fields[1].field_type,
            FieldType::Vec(Box::new(FieldType::String))
        );
        assert_eq!(
            fields[2].field_type,
            FieldType::Map(Box::new(FieldType::String))
        );
        assert_eq!(
            fields[3].field_type,
            FieldType::Option(Box::new(FieldType::String))
        );
        let generated = generate(&config);
        assert!(generated.source.contains("BridgeKeyType::Symbol"));
        assert!(generated.source.contains("pub tags: Vec<String>"));
        assert!(generated
            .source
            .contains("pub labels: BTreeMap<String, String>"));
        assert!(generated.source.contains("pub note: Option<String>"));
        Ok(())
    }

    #[test]
    fn parses_connector_style_mapped_fields() -> Result<()> {
        let config = Config::parse(
            "mapped = Booking\nclass = UserGlobals:OkzBooking\nfield = Booking.status | selector=status | return=Symbol\nfield = Booking.customer | selector=customer | return=Mapped<Customer>\n",
            None,
        )?;
        assert_eq!(config.classes[0].class_ref.dictionary, "UserGlobals");
        assert_eq!(config.classes[0].class_ref.class_name, "OkzBooking");
        let fields = &config.mapped[0].fields;
        assert_eq!(fields[0].selector.as_deref(), Some("status"));
        assert_eq!(fields[0].field_type, FieldType::String);
        assert_eq!(fields[0].return_type.as_deref(), Some("Symbol"));
        assert_eq!(fields[1].selector.as_deref(), Some("customer"));
        assert_eq!(
            fields[1].field_type,
            FieldType::Mapped("Customer".to_string())
        );
        assert_eq!(fields[1].return_type.as_deref(), Some("Mapped<Customer>"));

        let explain = explain(&config);
        assert!(explain.contains("selector=status"));
        assert!(explain.contains("return=Symbol"));
        let json = explain_json(&config);
        assert!(json.contains(r#""selector":"customer""#));
        assert!(json.contains(r#""return":"Symbol""#));
        Ok(())
    }

    #[test]
    fn rejects_non_string_map_keys() {
        let err = FieldType::parse("BTreeMap<Symbol, String>").unwrap_err();
        assert!(err.contains("key type must be String"));
    }

    #[test]
    fn parses_map_aliases_and_nested_map_values() {
        assert_eq!(
            FieldType::parse("Map<String, SmallInt>").unwrap(),
            FieldType::Map(Box::new(FieldType::SmallInt))
        );
        assert_eq!(
            FieldType::parse("Dictionary<String>").unwrap(),
            FieldType::Map(Box::new(FieldType::String))
        );
        assert_eq!(
            FieldType::parse("BTreeMap<String, Vec<String>>").unwrap(),
            FieldType::Map(Box::new(FieldType::Vec(Box::new(FieldType::String))))
        );
    }

    #[test]
    fn sample_mapping_config_uses_rust_type_name() {
        let source = sample_mapping_config("booking draft");
        assert!(source.contains("mapped = BookingDraft"));
        assert!(source.contains("field = BookingDraft.tags | type=Vec<String>"));
        assert!(source.contains("field = BookingDraft.labels | type=BTreeMap<String, String>"));
        assert!(source.contains("field = BookingDraft.note | type=Option<String>"));
    }

    #[test]
    fn infers_mapping_config_from_bridge_value_tree() -> Result<()> {
        let source = mapping_config_from_bridge_value(
            "booking draft",
            &BridgeValue::dictionary([
                ("name".to_string(), BridgeValue::from("Tariq")),
                ("amount".to_string(), BridgeValue::from(100_i64)),
                (
                    "customer".to_string(),
                    BridgeValue::keyed_dictionary([
                        (crate::BridgeKey::symbol("name"), BridgeValue::from("Tariq")),
                        (crate::BridgeKey::symbol("vip"), BridgeValue::from(true)),
                    ]),
                ),
                (
                    "items".to_string(),
                    BridgeValue::array([
                        BridgeValue::dictionary([
                            ("sku".to_string(), BridgeValue::from("A-1")),
                            ("quantity".to_string(), BridgeValue::from(2_i64)),
                        ]),
                        BridgeValue::dictionary([
                            ("sku".to_string(), BridgeValue::from("B-2")),
                            ("quantity".to_string(), BridgeValue::from(1_i64)),
                        ]),
                    ]),
                ),
                (
                    "state".to_string(),
                    BridgeValue::Symbol("ready".to_string()),
                ),
                ("note".to_string(), BridgeValue::Nil),
            ]),
        );

        assert!(source.contains("mapped = BookingDraft"));
        assert!(source.contains(
            "field = BookingDraft.customer | type=Mapped<BookingDraftCustomer> | key=customer | key_type=String"
        ));
        assert!(source.contains("mapped = BookingDraftCustomer"));
        assert!(source.contains(
            "field = BookingDraftCustomer.name | type=String | key=name | key_type=Symbol"
        ));
        assert!(source.contains(
            "field = BookingDraft.items | type=Vec<Mapped<BookingDraftItem>> | key=items | key_type=String"
        ));
        assert!(source.contains("mapped = BookingDraftItem"));
        assert!(source.contains(
            "field = BookingDraftItem.quantity | type=SmallInt | key=quantity | key_type=String"
        ));
        assert!(source.contains("field = BookingDraft.state | type=String"));
        assert!(source.contains("field = BookingDraft.note | type=Option<Oop>"));

        let config = Config::parse(&source, None)?;
        assert_eq!(config.mapped.len(), 3);
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
            mapped: config.mapped,
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
            mapped: Vec::new(),
        };
        let report = check(&config)?;
        assert!(!report.exists);
        assert!(!report.up_to_date);
        Ok(())
    }
}
