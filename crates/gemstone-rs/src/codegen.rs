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
        let mut doc = None;
        for option in parts {
            let Some((option_key, value)) = option.split_once('=') else {
                return Err(format!("field option must look like key=value: {option}"));
            };
            match option_key.trim() {
                "key" => key = value.trim().to_string(),
                "key_type" | "keyType" => key_type = FieldKeyType::parse(value.trim())?,
                "type" => field_type = FieldType::parse(value.trim())?,
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
            doc,
        })
    }
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
}

impl FieldType {
    fn parse(value: &str) -> std::result::Result<Self, String> {
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
        }
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

    fn arg_names(&self) -> Vec<String> {
        if self.args.is_empty() {
            inferred_selector_args(&self.selector)
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

pub fn explain(config: &Config) -> String {
    let mut out = String::new();
    out.push_str("gemstone-rs codegen explain\n");
    out.push_str(&format!("output: {}\n", config.output.display()));
    out.push_str("test_stubs: generated_surface_names_are_stable\n");
    out.push_str(&format!("classes: {}\n", config.classes.len()));
    for class in &config.classes {
        out.push_str(&format!(
            "  class: {} methods={}\n",
            class.class_ref.display_name(),
            class.methods.len()
        ));
        for method in &class.methods {
            let args = method.arg_names();
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
                "    field: {}.{} key={} key_type={} type={}\n",
                field.mapped_name,
                field.rust_name,
                field.key,
                field.key_type.config_name(),
                field.field_type.config_name()
            ));
        }
    }
    out
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
        mapped: Vec::new(),
    })
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
     mapped = BookingDraft | doc=A typed Rust payload stored under BridgeRoot.\n\
     field = BookingDraft.name | type=String | key=name\n\
     field = BookingDraft.amount | type=SmallInt | key=amount\n\
     field = BookingDraft.currency | type=String | key=currency\n\
     field = BookingDraft.tags | type=Vec<String> | key=tags\n"
}

pub fn sample_mapping_config(mapped: &str) -> String {
    let mapped = rust_type_name(mapped);
    format!(
        "mapped = {mapped} | doc=Typed payload stored under GemStoneRsBridgeRoot.\n\
         field = {mapped}.name | type=String | key=name | key_type=String\n\
         field = {mapped}.amount | type=SmallInt | key=amount | key_type=String\n\
         field = {mapped}.tags | type=Vec<String> | key=tags | key_type=String\n"
    )
}

fn generate_source(config: &Config) -> String {
    let mut source = String::new();
    source.push_str("// @generated by gemstone-rs codegen. Do not edit by hand.\n");
    source.push_str(
        "use gemstone_rs::{\n    BridgeDictionary, BridgeFieldRead, BridgeFieldWrite, BridgeKey, BridgeKeyType, BridgeMapped,\n    BridgeValue, Error, Oop, Result, Session, Value,\n};\n\n",
    );

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

fn test_stubs_source(config: &Config) -> String {
    let mut names = Vec::new();
    for class in &config.classes {
        let struct_name = class.class_ref.struct_name();
        if class.methods.is_empty() {
            names.push(struct_name);
        } else {
            for method in &class.methods {
                names.push(format!("{struct_name}::{}", method.fn_name()));
            }
        }
    }
    names.extend(config.mapped.iter().map(|mapped| mapped.name.clone()));

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
    fn generates_wrapper_source() -> Result<()> {
        let config = Config::parse(
            "class = Object\nmethod = Object>>printString | return=String | doc=Print the receiver.\nmethod = Object>>at:put: | args=key,value\n",
            None,
        )?;
        let generated = generate(&config);
        assert!(generated.source.contains("pub struct Object<'a>"));
        assert!(generated.source.contains("mod generated_code_tests"));
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
        assert!(explanation.contains("class: Object methods=1"));
        assert!(explanation.contains("method: Object>>printString args=[] return=String"));
        assert!(explanation.contains("mapped: BookingDraft fields=1"));
        assert!(explanation
            .contains("field: BookingDraft.amount key=amount key_type=Symbol type=SmallInt"));
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
            "mapped = BookingDraft\nfield = BookingDraft.customer | type=Mapped<Customer> | key=customer | key_type=Symbol\nfield = BookingDraft.tags | type=Vec<String> | key=tags\n",
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
        let generated = generate(&config);
        assert!(generated.source.contains("BridgeKeyType::Symbol"));
        assert!(generated.source.contains("pub tags: Vec<String>"));
        Ok(())
    }

    #[test]
    fn sample_mapping_config_uses_rust_type_name() {
        let source = sample_mapping_config("booking draft");
        assert!(source.contains("mapped = BookingDraft"));
        assert!(source.contains("field = BookingDraft.tags | type=Vec<String>"));
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
