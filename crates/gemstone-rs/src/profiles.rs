use crate::codegen;
use std::error::Error as StdError;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub type Result<T> = std::result::Result<T, Error>;

pub const PROFILE_KIND: &str = "gemstone-rs-explorer-codegen-profiles";
pub const DEFAULT_PROFILE_PATH: &str = "gemstone-rs.codegen-profiles.json";
pub const PROFILE_SCHEMA_PATH: &str = "schemas/gemstone-rs.codegen-profiles.schema.json";
pub const SAMPLE_PROFILE_SOURCE: &str = r#"{
  "kind": "gemstone-rs-explorer-codegen-profiles",
  "version": 1,
  "profiles": [
    {
      "name": "default",
      "config": "examples/codegen/gemstone-rs.codegen",
      "root": "",
      "mapped": "BookingDraft",
      "className": "Object"
    },
    {
      "name": "object-wrapper",
      "config": "examples/codegen/gemstone-rs.codegen",
      "root": "",
      "mapped": "ObjectWrapper",
      "className": "Object"
    },
    {
      "name": "bridge-mapping",
      "config": "examples/codegen/gemstone-rs.codegen",
      "root": "",
      "mapped": "BookingDraft",
      "className": "Object"
    }
  ]
}
"#;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationReport {
    pub profile_count: usize,
    pub profile_names: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectProfiles {
    pub kind: String,
    pub version: u32,
    pub profiles: Vec<CodegenProfile>,
}

impl ProjectProfiles {
    pub fn validation_report(&self) -> ValidationReport {
        ValidationReport {
            profile_count: self.profiles.len(),
            profile_names: self
                .profiles
                .iter()
                .map(|profile| profile.name.clone())
                .collect(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&CodegenProfile> {
        self.profiles.iter().find(|profile| profile.name == name)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodegenProfile {
    pub name: String,
    pub config: Option<String>,
    pub root: Option<String>,
    pub mapped: Option<String>,
    pub class_name: Option<String>,
}

impl CodegenProfile {
    pub fn resolved_config_path(&self) -> Result<PathBuf> {
        let config = self
            .config
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| {
                Error::schema(format!("profile {} does not define config", self.name))
            })?;
        let config_path = PathBuf::from(config);
        if config_path.is_absolute() {
            return Ok(config_path);
        }
        let root = self.root.as_deref().unwrap_or_default().trim();
        if root.is_empty() {
            Ok(config_path)
        } else {
            Ok(PathBuf::from(root).join(config_path))
        }
    }

    pub fn resolved_config_path_with_base_root(&self, base_root: Option<&Path>) -> Result<PathBuf> {
        let path = self.resolved_config_path()?;
        if path.is_absolute() {
            Ok(path)
        } else if let Some(base_root) = base_root {
            Ok(base_root.join(path))
        } else {
            Ok(path)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProfileCheckEntry {
    pub name: String,
    pub config: Option<PathBuf>,
    pub output: Option<PathBuf>,
    pub exists: bool,
    pub up_to_date: bool,
    pub error: Option<String>,
}

impl ProfileCheckEntry {
    pub fn ok(&self) -> bool {
        self.error.is_none() && self.up_to_date
    }

    pub fn status(&self) -> &'static str {
        if self.ok() {
            "ok"
        } else if self.error.is_some() {
            "error"
        } else {
            "stale"
        }
    }

    pub fn to_json(&self) -> String {
        format!(
            r#"{{"name":"{}","ok":{},"config":{},"output":{},"exists":{},"upToDate":{},"error":{}}}"#,
            escape_json(&self.name),
            self.ok(),
            optional_json_path(self.config.as_deref()),
            optional_json_path(self.output.as_deref()),
            self.exists,
            self.up_to_date,
            optional_json_string(self.error.as_deref())
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProfileCheckCounts {
    pub profile_count: usize,
    pub ok_count: usize,
    pub stale_count: usize,
    pub error_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProfileCheckReport {
    pub path: PathBuf,
    pub entries: Vec<ProfileCheckEntry>,
}

impl ProfileCheckReport {
    pub fn counts(&self) -> ProfileCheckCounts {
        let profile_count = self.entries.len();
        let ok_count = self.entries.iter().filter(|entry| entry.ok()).count();
        let error_count = self
            .entries
            .iter()
            .filter(|entry| entry.error.is_some())
            .count();
        let stale_count = profile_count.saturating_sub(ok_count + error_count);
        ProfileCheckCounts {
            profile_count,
            ok_count,
            stale_count,
            error_count,
        }
    }

    pub fn ok(&self) -> bool {
        self.entries.iter().all(ProfileCheckEntry::ok)
    }

    pub fn to_json(&self) -> String {
        let counts = self.counts();
        let profiles = self
            .entries
            .iter()
            .map(ProfileCheckEntry::to_json)
            .collect::<Vec<_>>()
            .join(",");
        format!(
            r#"{{"success":true,"ok":{},"path":"{}","profileFile":"{}","profileCount":{},"okCount":{},"staleCount":{},"errorCount":{},"profiles":[{}]}}"#,
            self.ok(),
            escape_json(&self.path.display().to_string()),
            escape_json(&self.path.display().to_string()),
            counts.profile_count,
            counts.ok_count,
            counts.stale_count,
            counts.error_count,
            profiles
        )
    }
}

pub fn check_file(path: impl AsRef<Path>, base_root: Option<&Path>) -> Result<ProfileCheckReport> {
    let path = path.as_ref();
    let project = load_file(path)?;
    Ok(check_project(path, &project, base_root))
}

pub fn check_project(
    path: impl AsRef<Path>,
    project: &ProjectProfiles,
    base_root: Option<&Path>,
) -> ProfileCheckReport {
    let entries = project
        .profiles
        .iter()
        .map(|profile| check_profile(profile, base_root))
        .collect();
    ProfileCheckReport {
        path: path.as_ref().to_path_buf(),
        entries,
    }
}

pub fn check_profile(profile: &CodegenProfile, base_root: Option<&Path>) -> ProfileCheckEntry {
    let mut entry = ProfileCheckEntry {
        name: profile.name.clone(),
        config: None,
        output: None,
        exists: false,
        up_to_date: false,
        error: None,
    };
    let config_path = match profile.resolved_config_path_with_base_root(base_root) {
        Ok(path) => path,
        Err(err) => {
            entry.error = Some(err.to_string());
            return entry;
        }
    };
    entry.config = Some(config_path.clone());
    let config = match codegen::Config::from_file(&config_path) {
        Ok(config) => config,
        Err(err) => {
            entry.error = Some(err.to_string());
            return entry;
        }
    };
    match codegen::check(&config) {
        Ok(report) => {
            entry.output = Some(report.output);
            entry.exists = report.exists;
            entry.up_to_date = report.up_to_date;
        }
        Err(err) => entry.error = Some(err.to_string()),
    }
    entry
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Schema(String),
}

impl Error {
    fn schema(message: impl Into<String>) -> Self {
        Self::Schema(message.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "{err}"),
            Self::Schema(message) => write!(f, "{message}"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::Schema(_) => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

pub fn validate_file(path: impl AsRef<Path>) -> Result<ValidationReport> {
    Ok(load_file(path)?.validation_report())
}

pub fn load_file(path: impl AsRef<Path>) -> Result<ProjectProfiles> {
    let source = fs::read_to_string(path)?;
    parse_source(&source)
}

pub fn sample_source() -> &'static str {
    SAMPLE_PROFILE_SOURCE
}

pub fn validate_source(source: &str) -> Result<ValidationReport> {
    Ok(parse_source(source)?.validation_report())
}

pub fn parse_source(source: &str) -> Result<ProjectProfiles> {
    let value = ProfileJsonParser::new(source)
        .parse()
        .map_err(|err| Error::schema(format!("profile JSON parse error: {err}")))?;
    let ProfileJson::Object(root) = value else {
        return Err(Error::schema("profile root must be a JSON object"));
    };

    for (field, _) in &root {
        match field.as_str() {
            "kind" | "version" | "profiles" => {}
            _ => return Err(Error::schema(format!("{field} is not supported"))),
        }
    }

    let kind = match profile_json_get(&root, "kind") {
        Some(ProfileJson::String(kind)) if kind == PROFILE_KIND => kind.clone(),
        Some(ProfileJson::String(_)) => {
            return Err(Error::schema(format!("kind must be {PROFILE_KIND}")));
        }
        Some(_) => return Err(Error::schema("kind must be a string")),
        None => return Err(Error::schema("missing kind")),
    };

    let version = match profile_json_get(&root, "version") {
        Some(ProfileJson::Number(version)) if version == "1" => 1,
        Some(ProfileJson::Number(_)) => return Err(Error::schema("version must be 1")),
        Some(_) => return Err(Error::schema("version must be a number")),
        None => return Err(Error::schema("missing version")),
    };

    let profiles = match profile_json_get(&root, "profiles") {
        Some(ProfileJson::Array(profiles)) => profiles,
        Some(_) => return Err(Error::schema("profiles must be an array")),
        None => return Err(Error::schema("missing profiles")),
    };

    let mut parsed_profiles = Vec::new();
    for (index, profile) in profiles.iter().enumerate() {
        let ProfileJson::Object(fields) = profile else {
            return Err(Error::schema(format!(
                "profiles[{index}] must be an object"
            )));
        };
        let profile = CodegenProfile::from_fields(index, fields)?;
        if parsed_profiles
            .iter()
            .any(|existing: &CodegenProfile| existing.name == profile.name)
        {
            return Err(Error::schema(format!(
                "profiles[{index}].name duplicates {}",
                profile.name
            )));
        }
        parsed_profiles.push(profile);
    }

    Ok(ProjectProfiles {
        kind,
        version,
        profiles: parsed_profiles,
    })
}

fn optional_json_path(value: Option<&Path>) -> String {
    optional_json_string(value.map(|path| path.display().to_string()).as_deref())
}

fn optional_json_string(value: Option<&str>) -> String {
    match value {
        Some(value) => format!(r#""{}""#, escape_json(value)),
        None => "null".to_string(),
    }
}

fn escape_json(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => escaped.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => escaped.push(ch),
        }
    }
    escaped
}

impl CodegenProfile {
    fn from_fields(index: usize, fields: &[(String, ProfileJson)]) -> Result<Self> {
        for (field, value) in fields {
            match field.as_str() {
                "name" | "config" | "root" | "mapped" | "className" => {}
                _ => {
                    return Err(Error::schema(format!(
                        "profiles[{index}].{field} is not supported"
                    )));
                }
            }
            if !matches!(value, ProfileJson::String(_)) {
                return Err(Error::schema(format!(
                    "profiles[{index}].{field} must be a string"
                )));
            }
        }

        let name = match profile_json_get(fields, "name") {
            Some(ProfileJson::String(name)) if !name.trim().is_empty() => name.clone(),
            Some(ProfileJson::String(_)) => {
                return Err(Error::schema(format!(
                    "profiles[{index}].name must not be empty"
                )));
            }
            Some(_) => {
                return Err(Error::schema(format!(
                    "profiles[{index}].name must be a string"
                )));
            }
            None => return Err(Error::schema(format!("profiles[{index}].name is required"))),
        };

        Ok(Self {
            name,
            config: optional_string_field(fields, "config"),
            root: optional_string_field(fields, "root"),
            mapped: optional_string_field(fields, "mapped"),
            class_name: optional_string_field(fields, "className"),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ProfileJson {
    Null,
    Bool,
    Number(String),
    String(String),
    Array(Vec<ProfileJson>),
    Object(Vec<(String, ProfileJson)>),
}

fn profile_json_get<'a>(object: &'a [(String, ProfileJson)], key: &str) -> Option<&'a ProfileJson> {
    object
        .iter()
        .find(|(candidate, _)| candidate == key)
        .map(|(_, value)| value)
}

fn optional_string_field(object: &[(String, ProfileJson)], key: &str) -> Option<String> {
    match profile_json_get(object, key) {
        Some(ProfileJson::String(value)) => Some(value.clone()),
        _ => None,
    }
}

struct ProfileJsonParser<'a> {
    chars: Vec<char>,
    index: usize,
    _marker: std::marker::PhantomData<&'a str>,
}

impl<'a> ProfileJsonParser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().collect(),
            index: 0,
            _marker: std::marker::PhantomData,
        }
    }

    fn parse(mut self) -> std::result::Result<ProfileJson, String> {
        let value = self.parse_value()?;
        self.skip_ws();
        if self.index != self.chars.len() {
            return Err(format!("unexpected trailing content at {}", self.index));
        }
        Ok(value)
    }

    fn parse_value(&mut self) -> std::result::Result<ProfileJson, String> {
        self.skip_ws();
        match self.peek() {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => self.parse_string().map(ProfileJson::String),
            Some('t') => {
                self.consume_literal("true")?;
                Ok(ProfileJson::Bool)
            }
            Some('f') => {
                self.consume_literal("false")?;
                Ok(ProfileJson::Bool)
            }
            Some('n') => {
                self.consume_literal("null")?;
                Ok(ProfileJson::Null)
            }
            Some(ch) if ch == '-' || ch.is_ascii_digit() => self.parse_number(),
            Some(ch) => Err(format!("unexpected character '{ch}' at {}", self.index)),
            None => Err("unexpected end of JSON".to_string()),
        }
    }

    fn parse_object(&mut self) -> std::result::Result<ProfileJson, String> {
        self.expect('{')?;
        let mut fields = Vec::new();
        self.skip_ws();
        if self.take_if('}') {
            return Ok(ProfileJson::Object(fields));
        }
        loop {
            self.skip_ws();
            let key = self.parse_string()?;
            self.skip_ws();
            self.expect(':')?;
            let value = self.parse_value()?;
            fields.push((key, value));
            self.skip_ws();
            if self.take_if('}') {
                break;
            }
            self.expect(',')?;
        }
        Ok(ProfileJson::Object(fields))
    }

    fn parse_array(&mut self) -> std::result::Result<ProfileJson, String> {
        self.expect('[')?;
        let mut values = Vec::new();
        self.skip_ws();
        if self.take_if(']') {
            return Ok(ProfileJson::Array(values));
        }
        loop {
            values.push(self.parse_value()?);
            self.skip_ws();
            if self.take_if(']') {
                break;
            }
            self.expect(',')?;
        }
        Ok(ProfileJson::Array(values))
    }

    fn parse_string(&mut self) -> std::result::Result<String, String> {
        self.expect('"')?;
        let mut value = String::new();
        while let Some(ch) = self.next() {
            match ch {
                '"' => return Ok(value),
                '\\' => {
                    let escaped = self
                        .next()
                        .ok_or_else(|| "unterminated string escape".to_string())?;
                    match escaped {
                        '"' | '\\' | '/' => value.push(escaped),
                        'b' => value.push('\u{0008}'),
                        'f' => value.push('\u{000c}'),
                        'n' => value.push('\n'),
                        'r' => value.push('\r'),
                        't' => value.push('\t'),
                        'u' => {
                            let mut code = String::new();
                            for _ in 0..4 {
                                let Some(hex) = self.next() else {
                                    return Err("incomplete unicode escape".to_string());
                                };
                                if !hex.is_ascii_hexdigit() {
                                    return Err("invalid unicode escape".to_string());
                                }
                                code.push(hex);
                            }
                            if let Ok(codepoint) = u32::from_str_radix(&code, 16) {
                                if let Some(decoded) = char::from_u32(codepoint) {
                                    value.push(decoded);
                                }
                            }
                        }
                        other => return Err(format!("invalid string escape \\{other}")),
                    }
                }
                ch if ch.is_control() => {
                    return Err("unescaped control character in string".to_string());
                }
                other => value.push(other),
            }
        }
        Err("unterminated string".to_string())
    }

    fn parse_number(&mut self) -> std::result::Result<ProfileJson, String> {
        let start = self.index;
        if self.peek() == Some('-') {
            self.index += 1;
        }
        self.consume_digits();
        if self.peek() == Some('.') {
            self.index += 1;
            self.consume_digits();
        }
        if matches!(self.peek(), Some('e' | 'E')) {
            self.index += 1;
            if matches!(self.peek(), Some('+' | '-')) {
                self.index += 1;
            }
            self.consume_digits();
        }
        if self.index == start {
            return Err(format!("expected number at {start}"));
        }
        Ok(ProfileJson::Number(
            self.chars[start..self.index].iter().collect(),
        ))
    }

    fn consume_digits(&mut self) {
        while matches!(self.peek(), Some(ch) if ch.is_ascii_digit()) {
            self.index += 1;
        }
    }

    fn consume_literal(&mut self, literal: &str) -> std::result::Result<(), String> {
        for expected in literal.chars() {
            self.expect(expected)?;
        }
        Ok(())
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(ch) if ch.is_whitespace()) {
            self.index += 1;
        }
    }

    fn expect(&mut self, expected: char) -> std::result::Result<(), String> {
        match self.next() {
            Some(ch) if ch == expected => Ok(()),
            Some(ch) => Err(format!(
                "expected '{expected}' at {}, got '{ch}'",
                self.index.saturating_sub(1)
            )),
            None => Err(format!("expected '{expected}', got end of JSON")),
        }
    }

    fn take_if(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.index).copied()
    }

    fn next(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.index += 1;
        Some(ch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = r#"{"kind":"gemstone-rs-explorer-codegen-profiles","version":1,"profiles":[{"name":"default","config":"examples/codegen/gemstone-rs.codegen","root":"","mapped":"BookingDraft","className":"Object"}]}"#;

    #[test]
    fn validates_profile_file_shape() {
        let report = validate_source(VALID).unwrap();
        assert_eq!(report.profile_count, 1);
        assert_eq!(report.profile_names, vec!["default".to_string()]);
    }

    #[test]
    fn parses_project_profiles() {
        let project = parse_source(VALID).unwrap();
        assert_eq!(project.kind, PROFILE_KIND);
        assert_eq!(project.version, 1);
        assert_eq!(project.profiles.len(), 1);
        assert_eq!(
            project.get("default"),
            Some(&CodegenProfile {
                name: "default".to_string(),
                config: Some("examples/codegen/gemstone-rs.codegen".to_string()),
                root: Some("".to_string()),
                mapped: Some("BookingDraft".to_string()),
                class_name: Some("Object".to_string()),
            })
        );
        assert!(project.get("missing").is_none());
    }

    #[test]
    fn resolves_profile_config_path() {
        let profile = CodegenProfile {
            name: "default".to_string(),
            config: Some("examples/codegen/gemstone-rs.codegen".to_string()),
            root: Some("workspace".to_string()),
            mapped: None,
            class_name: None,
        };
        assert_eq!(
            profile.resolved_config_path().unwrap(),
            PathBuf::from("workspace/examples/codegen/gemstone-rs.codegen")
        );

        let profile = CodegenProfile {
            name: "default".to_string(),
            config: Some("/tmp/gemstone-rs.codegen".to_string()),
            root: Some("workspace".to_string()),
            mapped: None,
            class_name: None,
        };
        assert_eq!(
            profile.resolved_config_path().unwrap(),
            PathBuf::from("/tmp/gemstone-rs.codegen")
        );

        let profile = CodegenProfile {
            name: "default".to_string(),
            config: Some("examples/codegen/gemstone-rs.codegen".to_string()),
            root: Some("workspace".to_string()),
            mapped: None,
            class_name: None,
        };
        assert_eq!(
            profile
                .resolved_config_path_with_base_root(Some(Path::new("/repo")))
                .unwrap(),
            PathBuf::from("/repo/workspace/examples/codegen/gemstone-rs.codegen")
        );
    }

    #[test]
    fn profile_check_report_counts_and_json_are_shared() {
        let report = ProfileCheckReport {
            path: PathBuf::from("profiles.json"),
            entries: vec![
                ProfileCheckEntry {
                    name: "fresh".to_string(),
                    config: Some(PathBuf::from("fresh.codegen")),
                    output: Some(PathBuf::from("fresh.rs")),
                    exists: true,
                    up_to_date: true,
                    error: None,
                },
                ProfileCheckEntry {
                    name: "stale".to_string(),
                    config: Some(PathBuf::from("stale.codegen")),
                    output: Some(PathBuf::from("stale.rs")),
                    exists: true,
                    up_to_date: false,
                    error: None,
                },
                ProfileCheckEntry {
                    name: "broken".to_string(),
                    config: None,
                    output: None,
                    exists: false,
                    up_to_date: false,
                    error: Some("missing config".to_string()),
                },
            ],
        };
        assert_eq!(
            report.counts(),
            ProfileCheckCounts {
                profile_count: 3,
                ok_count: 1,
                stale_count: 1,
                error_count: 1
            }
        );
        let json = report.to_json();
        assert!(json.contains(r#""success":true"#));
        assert!(json.contains(r#""ok":false"#));
        assert!(json.contains(r#""profileFile":"profiles.json""#));
        assert!(json.contains(r#""exists":true"#));
        assert!(json.contains(r#""upToDate":false"#));
        assert!(json.contains(r#""error":"missing config""#));
    }

    #[test]
    fn reports_field_level_schema_errors() {
        assert_eq!(
            validate_source(r#"{"kind":"bad","version":1,"profiles":[]}"#)
                .unwrap_err()
                .to_string(),
            "kind must be gemstone-rs-explorer-codegen-profiles"
        );
        assert_eq!(
            validate_source(
                r#"{"kind":"gemstone-rs-explorer-codegen-profiles","version":1,"profiles":[{}]}"#
            )
            .unwrap_err()
            .to_string(),
            "profiles[0].name is required"
        );
        assert_eq!(
            validate_source(
                r#"{"kind":"gemstone-rs-explorer-codegen-profiles","version":1,"extra":true,"profiles":[]}"#
            )
            .unwrap_err()
            .to_string(),
            "extra is not supported"
        );
        assert_eq!(
            validate_source(
                r#"{"kind":"gemstone-rs-explorer-codegen-profiles","version":1,"profiles":[{"name":"default","unsupported":"value"}]}"#
            )
            .unwrap_err()
            .to_string(),
            "profiles[0].unsupported is not supported"
        );
        assert_eq!(
            validate_source(
                r#"{"kind":"gemstone-rs-explorer-codegen-profiles","version":1,"profiles":[{"name":"default"},{"name":"default"}]}"#
            )
            .unwrap_err()
            .to_string(),
            "profiles[1].name duplicates default"
        );
    }

    #[test]
    fn sample_source_is_valid() {
        let report = validate_source(sample_source()).unwrap();
        assert_eq!(report.profile_count, 3);
        assert_eq!(
            report.profile_names,
            vec![
                "default".to_string(),
                "object-wrapper".to_string(),
                "bridge-mapping".to_string()
            ]
        );
    }
}
