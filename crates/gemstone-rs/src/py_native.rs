//! Stable adapter surface for `gemstone-py-native` style PyO3 wrappers.
//!
//! This module intentionally has no PyO3 dependency. It exposes plain Rust
//! structs and enums that are easy for a PyO3 crate to wrap while keeping the
//! actual GemStone/S GCI implementation in `gemstone-gci` and `gemstone-rs`.
//! The goal is a narrow contract:
//!
//! ```text
//! Python -> PyO3 classes/functions -> gemstone_rs::py_native -> Session -> GCI
//! ```
//!
//! `PyNativeSession` is deliberately synchronous and thread-conservative. A
//! Python package can layer Pythonic sync/async APIs above it without taking
//! ownership of native loading, OOP conversion, login/logout, or transaction
//! behavior.

use crate::{Config, Error, Oop, Result, Session, Value};
use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyNativeConfig {
    pub stone: String,
    pub netldi: String,
    pub host: String,
    pub username: String,
    pub password: String,
    pub host_username: String,
    pub host_password: String,
    pub gem_service: String,
    pub lib_path: Option<PathBuf>,
}

impl PyNativeConfig {
    pub fn from_env() -> Result<Self> {
        Config::from_env().map(Self::from_config)
    }

    pub fn from_config(config: Config) -> Self {
        Self {
            stone: config.stone,
            netldi: config.netldi,
            host: config.host,
            username: config.username,
            password: config.password,
            host_username: config.host_username,
            host_password: config.host_password,
            gem_service: config.gem_service,
            lib_path: config.lib_path,
        }
    }

    pub fn into_config(self) -> Result<Config> {
        let mut builder = Config::builder()
            .stone(self.stone)
            .netldi(self.netldi)
            .host(self.host)
            .username(self.username)
            .password(self.password)
            .host_username(self.host_username)
            .host_password(self.host_password)
            .gem_service(self.gem_service);
        if let Some(lib_path) = self.lib_path {
            builder = builder.lib_path(lib_path);
        }
        builder.build()
    }

    pub fn redacted_summary(&self) -> PyNativeConfigSummary {
        PyNativeConfigSummary {
            stone: self.stone.clone(),
            netldi: self.netldi.clone(),
            host: self.host.clone(),
            username: self.username.clone(),
            host_username: self.host_username.clone(),
            gem_service: self.gem_service.clone(),
            lib_path: self.lib_path.clone(),
            password_set: !self.password.is_empty(),
            host_password_set: !self.host_password.is_empty(),
        }
    }
}

impl Default for PyNativeConfig {
    fn default() -> Self {
        Self::from_config(Config::default())
    }
}

impl From<Config> for PyNativeConfig {
    fn from(value: Config) -> Self {
        Self::from_config(value)
    }
}

impl TryFrom<PyNativeConfig> for Config {
    type Error = Error;

    fn try_from(value: PyNativeConfig) -> Result<Self> {
        value.into_config()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyNativeConfigSummary {
    pub stone: String,
    pub netldi: String,
    pub host: String,
    pub username: String,
    pub host_username: String,
    pub gem_service: String,
    pub lib_path: Option<PathBuf>,
    pub password_set: bool,
    pub host_password_set: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PyNativeValue {
    Nil,
    Bool(bool),
    SmallInt(i64),
    Char(char),
    String(String),
    Symbol(String),
    Oop(u64),
}

impl PyNativeValue {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Nil => "nil",
            Self::Bool(_) => "bool",
            Self::SmallInt(_) => "smallInt",
            Self::Char(_) => "char",
            Self::String(_) => "string",
            Self::Symbol(_) => "symbol",
            Self::Oop(_) => "oop",
        }
    }

    pub fn raw_oop(&self) -> Option<u64> {
        match self {
            Self::Oop(raw) => Some(*raw),
            _ => None,
        }
    }

    pub fn from_value(value: Value) -> Self {
        match value {
            Value::Nil => Self::Nil,
            Value::Bool(value) => Self::Bool(value),
            Value::SmallInt(value) => Self::SmallInt(value),
            Value::Char(value) => Self::Char(value),
            Value::String(value) => Self::String(value),
            Value::Oop(oop) => Self::Oop(oop.raw()),
        }
    }

    pub fn to_value(&self) -> Option<Value> {
        match self {
            Self::Nil => Some(Value::Nil),
            Self::Bool(value) => Some(Value::Bool(*value)),
            Self::SmallInt(value) => Some(Value::SmallInt(*value)),
            Self::Char(value) => Some(Value::Char(*value)),
            Self::String(value) => Some(Value::String(value.clone())),
            Self::Symbol(_) => None,
            Self::Oop(raw) => Some(Value::Oop(Oop(*raw))),
        }
    }

    pub fn to_oop(&self, session: &mut Session) -> Result<Oop> {
        match self {
            Self::Nil => Ok(Oop::NIL),
            Self::Bool(value) => Ok(Oop::from_bool(*value)),
            Self::SmallInt(value) => Ok(Oop::from_smallint(*value)),
            Self::Char(value) => Ok(Oop::from_char(*value)),
            Self::String(value) => session.new_string(value),
            Self::Symbol(value) => session.new_symbol(value),
            Self::Oop(raw) => Ok(Oop(*raw)),
        }
    }

    pub fn to_json(&self) -> String {
        match self {
            Self::Nil => r#"{"kind":"nil"}"#.to_string(),
            Self::Bool(value) => format!(
                r#"{{"kind":"bool","value":{}}}"#,
                if *value { "true" } else { "false" }
            ),
            Self::SmallInt(value) => format!(r#"{{"kind":"smallInt","value":{value}}}"#),
            Self::Char(value) => format!(
                r#"{{"kind":"char","value":"{}"}}"#,
                json_escape(&value.to_string())
            ),
            Self::String(value) => {
                format!(r#"{{"kind":"string","value":"{}"}}"#, json_escape(value))
            }
            Self::Symbol(value) => {
                format!(r#"{{"kind":"symbol","value":"{}"}}"#, json_escape(value))
            }
            Self::Oop(raw) => format!(r#"{{"kind":"oop","raw":{raw}}}"#),
        }
    }
}

impl From<Value> for PyNativeValue {
    fn from(value: Value) -> Self {
        Self::from_value(value)
    }
}

impl From<Oop> for PyNativeValue {
    fn from(value: Oop) -> Self {
        Self::Oop(value.raw())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PyNativeErrorKind {
    Gci,
    MissingEnvironment,
    MissingConfig,
    Nul,
    NotLoggedIn,
    GemStone,
    IllegalOop,
    UnexpectedType,
    Mapping,
    WorkerStopped,
    WorkerPanicked,
    NegativeSize,
    ArgumentCountTooLarge,
}

impl PyNativeErrorKind {
    pub fn as_json_name(self) -> &'static str {
        match self {
            Self::Gci => "gci",
            Self::MissingEnvironment => "missingEnvironment",
            Self::MissingConfig => "missingConfig",
            Self::Nul => "nul",
            Self::NotLoggedIn => "notLoggedIn",
            Self::GemStone => "gemStone",
            Self::IllegalOop => "illegalOop",
            Self::UnexpectedType => "unexpectedType",
            Self::Mapping => "mapping",
            Self::WorkerStopped => "workerStopped",
            Self::WorkerPanicked => "workerPanicked",
            Self::NegativeSize => "negativeSize",
            Self::ArgumentCountTooLarge => "argumentCountTooLarge",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyNativeErrorInfo {
    pub kind: PyNativeErrorKind,
    pub message: String,
    pub gemstone_number: Option<i32>,
    pub fatal: Option<bool>,
    pub operation: Option<&'static str>,
    pub field: Option<String>,
    pub expected: Option<&'static str>,
    pub actual: Option<String>,
}

impl PyNativeErrorInfo {
    pub fn from_error(error: &Error) -> Self {
        match error {
            Error::Gci(_) => Self::simple(PyNativeErrorKind::Gci, error),
            Error::MissingEnvironment(_) => {
                Self::simple(PyNativeErrorKind::MissingEnvironment, error)
            }
            Error::MissingConfig(_) => Self::simple(PyNativeErrorKind::MissingConfig, error),
            Error::Nul(_) => Self::simple(PyNativeErrorKind::Nul, error),
            Error::NotLoggedIn => Self::simple(PyNativeErrorKind::NotLoggedIn, error),
            Error::GemStone {
                number,
                fatal,
                message,
            } => Self {
                kind: PyNativeErrorKind::GemStone,
                message: message.clone(),
                gemstone_number: Some(*number),
                fatal: Some(*fatal),
                operation: None,
                field: None,
                expected: None,
                actual: None,
            },
            Error::IllegalOop { operation } => Self {
                kind: PyNativeErrorKind::IllegalOop,
                message: error.to_string(),
                gemstone_number: None,
                fatal: None,
                operation: Some(*operation),
                field: None,
                expected: None,
                actual: None,
            },
            Error::UnexpectedType { expected, actual } => Self {
                kind: PyNativeErrorKind::UnexpectedType,
                message: error.to_string(),
                gemstone_number: None,
                fatal: None,
                operation: None,
                field: None,
                expected: Some(*expected),
                actual: Some(actual.clone()),
            },
            Error::Mapping {
                field,
                expected,
                actual,
            } => Self {
                kind: PyNativeErrorKind::Mapping,
                message: error.to_string(),
                gemstone_number: None,
                fatal: None,
                operation: None,
                field: Some(field.clone()),
                expected: Some(*expected),
                actual: Some(actual.clone()),
            },
            Error::WorkerStopped => Self::simple(PyNativeErrorKind::WorkerStopped, error),
            Error::WorkerPanicked => Self::simple(PyNativeErrorKind::WorkerPanicked, error),
            Error::NegativeSize(_) => Self::simple(PyNativeErrorKind::NegativeSize, error),
            Error::ArgumentCountTooLarge(_) => {
                Self::simple(PyNativeErrorKind::ArgumentCountTooLarge, error)
            }
        }
    }

    fn simple(kind: PyNativeErrorKind, error: &Error) -> Self {
        Self {
            kind,
            message: error.to_string(),
            gemstone_number: None,
            fatal: None,
            operation: None,
            field: None,
            expected: None,
            actual: None,
        }
    }

    pub fn to_json(&self) -> String {
        let mut fields = vec![
            format!(r#""kind":"{}""#, self.kind.as_json_name()),
            format!(r#""message":"{}""#, json_escape(&self.message)),
        ];
        if let Some(number) = self.gemstone_number {
            fields.push(format!(r#""gemstoneNumber":{number}"#));
        }
        if let Some(fatal) = self.fatal {
            fields.push(format!(
                r#""fatal":{}"#,
                if fatal { "true" } else { "false" }
            ));
        }
        if let Some(operation) = self.operation {
            fields.push(format!(r#""operation":"{}""#, json_escape(operation)));
        }
        if let Some(field) = &self.field {
            fields.push(format!(r#""field":"{}""#, json_escape(field)));
        }
        if let Some(expected) = self.expected {
            fields.push(format!(r#""expected":"{}""#, json_escape(expected)));
        }
        if let Some(actual) = &self.actual {
            fields.push(format!(r#""actual":"{}""#, json_escape(actual)));
        }
        format!("{{{}}}", fields.join(","))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyNativeCapabilities {
    pub contract_version: u16,
    pub threading: &'static str,
    pub operations: &'static [&'static str],
}

impl PyNativeCapabilities {
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"name":"{}","contractVersion":{},"threading":"{}","operations":[{}],"valueKinds":[{}],"errorKinds":[{}],"oopConstants":{{"nil":{},"true":{},"false":{},"smallint7":{},"charA":{}}}}}"#,
            json_escape(PY_NATIVE_CONTRACT_NAME),
            self.contract_version,
            json_escape(self.threading),
            json_string_array(self.operations),
            json_string_array(PY_NATIVE_VALUE_KINDS),
            json_string_array(PY_NATIVE_ERROR_KINDS),
            nil_oop(),
            bool_oop(true),
            bool_oop(false),
            smallint_oop(7),
            char_oop('A')
        )
    }
}

pub const PY_NATIVE_CONTRACT_NAME: &str = "gemstone-py-native adapter contract";

pub const PY_NATIVE_OPERATIONS: &[&str] = &[
    "login",
    "logout",
    "eval",
    "eval_oop",
    "execute",
    "resolve",
    "value_to_oop",
    "perform",
    "new_string",
    "new_symbol",
    "fetch_string",
    "global_get",
    "global_put",
    "commit",
    "abort",
    "needs_commit",
    "in_transaction",
    "add_to_export_set",
    "remove_from_export_set",
];

pub const PY_NATIVE_VALUE_KINDS: &[&str] =
    &["nil", "bool", "smallInt", "char", "string", "symbol", "oop"];

pub const PY_NATIVE_ERROR_KINDS: &[&str] = &[
    "gci",
    "missingEnvironment",
    "missingConfig",
    "nul",
    "notLoggedIn",
    "gemStone",
    "illegalOop",
    "unexpectedType",
    "mapping",
    "workerStopped",
    "workerPanicked",
    "negativeSize",
    "argumentCountTooLarge",
];

pub fn capabilities() -> PyNativeCapabilities {
    PyNativeCapabilities {
        contract_version: 1,
        threading: "session is synchronous and non-Send/non-Sync; PyO3 wrappers should use unsendable classes or dedicated worker threads",
        operations: PY_NATIVE_OPERATIONS,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyNativeMigrationStep {
    pub id: &'static str,
    pub title: &'static str,
    pub status: &'static str,
    pub detail: &'static str,
    pub verify: &'static str,
}

impl PyNativeMigrationStep {
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"id":"{}","title":"{}","status":"{}","detail":"{}","verify":"{}"}}"#,
            json_escape(self.id),
            json_escape(self.title),
            json_escape(self.status),
            json_escape(self.detail),
            json_escape(self.verify)
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyNativeMigrationReport {
    pub contract_version: u16,
    pub target_package: &'static str,
    pub status: &'static str,
    pub steps: Vec<PyNativeMigrationStep>,
}

impl PyNativeMigrationReport {
    pub fn done_count(&self) -> usize {
        self.steps
            .iter()
            .filter(|step| step.status == "done")
            .count()
    }

    pub fn pending_count(&self) -> usize {
        self.steps
            .iter()
            .filter(|step| step.status == "pending")
            .count()
    }

    pub fn to_json(&self) -> String {
        let steps = self
            .steps
            .iter()
            .map(PyNativeMigrationStep::to_json)
            .collect::<Vec<_>>()
            .join(",");
        format!(
            r#"{{"contractVersion":{},"targetPackage":"{}","status":"{}","doneCount":{},"pendingCount":{},"steps":[{}]}}"#,
            self.contract_version,
            json_escape(self.target_package),
            json_escape(self.status),
            self.done_count(),
            self.pending_count(),
            steps
        )
    }
}

pub fn migration_report() -> PyNativeMigrationReport {
    PyNativeMigrationReport {
        contract_version: capabilities().contract_version,
        target_package: "gemstone-py-native",
        status: "Rust adapter contract is wired into gemstone-py-native; local live smoke and TestPyPI/PyPI published wheel verification have passed",
        steps: vec![
            PyNativeMigrationStep {
                id: "scaffold_pyo3_adapter",
                title: "Keep the PyO3 starter scaffold current",
                status: "done",
                detail: "gemstone-rs can scaffold and verify a thin PyO3 adapter over gemstone_rs::py_native.",
                verify: "python3 scripts/check_py_native_pyo3_scaffold.py",
            },
            PyNativeMigrationStep {
                id: "wrap_py_native_session",
                title: "Wrap PyNativeSession in gemstone-py-native",
                status: "done",
                detail: "gemstone-py-native exposes an additive RustCoreSession PyO3 class and rust_core_* reports that delegate login, eval, perform, OOP conversion, globals, transactions, and export-set calls to gemstone_rs::py_native.",
                verify: "gemstone-py tests/test_native_crate.py",
            },
            PyNativeMigrationStep {
                id: "preserve_python_api",
                title: "Preserve existing Python return behavior",
                status: "done",
                detail: "The Rust-backed surface is additive, so existing gemstone-py sync APIs keep their current Session.execute()/perform() behavior while Rust-managed handles remain opt-in/native-path behavior.",
                verify: "gemstone-py tests/test_native_crate.py plus existing sync/async tests",
            },
            PyNativeMigrationStep {
                id: "live_backend_smoke",
                title: "Run live GemStone smoke through the Rust-backed native path",
                status: "done",
                detail: "The downstream gemstone-py live smoke validates login/logout, 3 + 4, perform, globals, commit/abort, and lifetime/export-set behavior against a real stone through the Python package.",
                verify: "GS_RUN_LIVE=1 python scripts/run_native_rust_core_live_smoke.py --require-live",
            },
            PyNativeMigrationStep {
                id: "publish_wheels",
                title: "Publish wheels after live smoke is green",
                status: "done",
                detail: "gemstone-py-native 0.1.3 wheels and sdist were published with trusted publishing and verified from TestPyPI and PyPI while using gemstone-rs as the shared Rust core.",
                verify: "native-wheels workflow TestPyPI/PyPI install plus native backend verification",
            },
        ],
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyNativeCompatibilityMethod {
    pub python_method: &'static str,
    pub native_method: &'static str,
    pub native_return: &'static str,
    pub python_return: &'static str,
    pub note: &'static str,
}

impl PyNativeCompatibilityMethod {
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"pythonMethod":"{}","nativeMethod":"{}","nativeReturn":"{}","pythonReturn":"{}","note":"{}"}}"#,
            json_escape(self.python_method),
            json_escape(self.native_method),
            json_escape(self.native_return),
            json_escape(self.python_return),
            json_escape(self.note)
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyNativeCompatibilityReport {
    pub contract_version: u16,
    pub module: &'static str,
    pub session_class: &'static str,
    pub handle_class: &'static str,
    pub return_policy: &'static str,
    pub methods: Vec<PyNativeCompatibilityMethod>,
}

impl PyNativeCompatibilityReport {
    pub fn to_json(&self) -> String {
        let methods = self
            .methods
            .iter()
            .map(PyNativeCompatibilityMethod::to_json)
            .collect::<Vec<_>>()
            .join(",");
        format!(
            r#"{{"contractVersion":{},"module":"{}","sessionClass":"{}","handleClass":"{}","returnPolicy":"{}","methods":[{}]}}"#,
            self.contract_version,
            json_escape(self.module),
            json_escape(self.session_class),
            json_escape(self.handle_class),
            json_escape(self.return_policy),
            methods
        )
    }
}

pub fn compatibility_report() -> PyNativeCompatibilityReport {
    PyNativeCompatibilityReport {
        contract_version: capabilities().contract_version,
        module: "gemstone_py_native_compat",
        session_class: "NativeCompatibilitySession",
        handle_class: "OopHandle",
        return_policy: "object identity returns OopHandle, raw native OOPs stay below the package boundary, and typed helpers are opt-in",
        methods: vec![
            PyNativeCompatibilityMethod {
                python_method: "login_from_env",
                native_method: "NativeSession.login_from_env",
                native_return: "NativeSession",
                python_return: "NativeCompatibilitySession",
                note: "constructs the package-level compatibility wrapper",
            },
            PyNativeCompatibilityMethod {
                python_method: "session_id",
                native_method: "NativeSession.session_id",
                native_return: "int",
                python_return: "int",
                note: "passes the native session id through unchanged",
            },
            PyNativeCompatibilityMethod {
                python_method: "eval_repr",
                native_method: "NativeSession.eval_repr",
                native_return: "str",
                python_return: "str",
                note: "debug representation helper for diagnostics",
            },
            PyNativeCompatibilityMethod {
                python_method: "eval_value",
                native_method: "NativeSession.eval_json",
                native_return: "str",
                python_return: "dict",
                note: "returns the stable PyNativeValue JSON shape as a Python dictionary",
            },
            PyNativeCompatibilityMethod {
                python_method: "eval_smallint",
                native_method: "NativeSession.eval_smallint",
                native_return: "int",
                python_return: "int",
                note: "typed helper remains explicit opt-in",
            },
            PyNativeCompatibilityMethod {
                python_method: "eval_oop",
                native_method: "NativeSession.eval_oop",
                native_return: "u64",
                python_return: "OopHandle",
                note: "wraps raw object identity before package callers see it",
            },
            PyNativeCompatibilityMethod {
                python_method: "execute",
                native_method: "NativeSession.execute",
                native_return: "u64",
                python_return: "OopHandle",
                note: "wraps raw object identity before package callers see it",
            },
            PyNativeCompatibilityMethod {
                python_method: "resolve",
                native_method: "NativeSession.resolve",
                native_return: "u64",
                python_return: "OopHandle",
                note: "wraps resolved globals/classes as handles",
            },
            PyNativeCompatibilityMethod {
                python_method: "perform_oop",
                native_method: "NativeSession.perform_raw_oop",
                native_return: "u64",
                python_return: "OopHandle",
                note: "accepts OopHandle or int arguments and returns a handle",
            },
            PyNativeCompatibilityMethod {
                python_method: "perform_value",
                native_method: "NativeSession.perform_json",
                native_return: "str",
                python_return: "dict",
                note: "accepts OopHandle or int arguments and returns the stable PyNativeValue JSON shape as a Python dictionary",
            },
            PyNativeCompatibilityMethod {
                python_method: "new_string",
                native_method: "NativeSession.new_string",
                native_return: "u64",
                python_return: "OopHandle",
                note: "returns the GemStone string object identity as a handle",
            },
            PyNativeCompatibilityMethod {
                python_method: "new_symbol",
                native_method: "NativeSession.new_symbol",
                native_return: "u64",
                python_return: "OopHandle",
                note: "returns the GemStone symbol object identity as a handle",
            },
            PyNativeCompatibilityMethod {
                python_method: "fetch_string",
                native_method: "NativeSession.fetch_string",
                native_return: "str",
                python_return: "str",
                note: "accepts OopHandle or int and returns a Python string",
            },
            PyNativeCompatibilityMethod {
                python_method: "global_get",
                native_method: "NativeSession.global_get",
                native_return: "u64",
                python_return: "OopHandle",
                note: "wraps UserGlobals values as handles",
            },
            PyNativeCompatibilityMethod {
                python_method: "global_put_oop",
                native_method: "NativeSession.global_put_raw",
                native_return: "None",
                python_return: "None",
                note: "accepts OopHandle or int and writes the raw OOP below the package boundary",
            },
            PyNativeCompatibilityMethod {
                python_method: "global_put_string",
                native_method: "NativeSession.global_put_string",
                native_return: "None",
                python_return: "None",
                note: "typed helper remains explicit opt-in",
            },
            PyNativeCompatibilityMethod {
                python_method: "global_put_smallint",
                native_method: "NativeSession.global_put_smallint",
                native_return: "None",
                python_return: "None",
                note: "typed helper remains explicit opt-in",
            },
            PyNativeCompatibilityMethod {
                python_method: "value_to_oop_nil",
                native_method: "NativeSession.value_to_oop_nil",
                native_return: "u64",
                python_return: "OopHandle",
                note: "explicit typed conversion helper for nil",
            },
            PyNativeCompatibilityMethod {
                python_method: "value_to_oop_bool",
                native_method: "NativeSession.value_to_oop_bool",
                native_return: "u64",
                python_return: "OopHandle",
                note: "explicit typed conversion helper for booleans",
            },
            PyNativeCompatibilityMethod {
                python_method: "value_to_oop_smallint",
                native_method: "NativeSession.value_to_oop_smallint",
                native_return: "u64",
                python_return: "OopHandle",
                note: "explicit typed conversion helper for small integers",
            },
            PyNativeCompatibilityMethod {
                python_method: "value_to_oop_char",
                native_method: "NativeSession.value_to_oop_char",
                native_return: "u64",
                python_return: "OopHandle",
                note: "explicit typed conversion helper for single-character strings",
            },
            PyNativeCompatibilityMethod {
                python_method: "value_to_oop_string",
                native_method: "NativeSession.value_to_oop_string",
                native_return: "u64",
                python_return: "OopHandle",
                note: "explicit typed conversion helper for GemStone strings",
            },
            PyNativeCompatibilityMethod {
                python_method: "value_to_oop_symbol",
                native_method: "NativeSession.value_to_oop_symbol",
                native_return: "u64",
                python_return: "OopHandle",
                note: "explicit typed conversion helper for GemStone symbols",
            },
            PyNativeCompatibilityMethod {
                python_method: "value_to_oop_raw",
                native_method: "NativeSession.value_to_oop_raw",
                native_return: "u64",
                python_return: "OopHandle",
                note: "preserves raw OOP identity while wrapping it at the Python package boundary",
            },
            PyNativeCompatibilityMethod {
                python_method: "add_to_export_set",
                native_method: "NativeSession.add_to_export_set",
                native_return: "None",
                python_return: "None",
                note: "accepts OopHandle or int for lifetime retention",
            },
            PyNativeCompatibilityMethod {
                python_method: "remove_from_export_set",
                native_method: "NativeSession.remove_from_export_set",
                native_return: "None",
                python_return: "None",
                note: "accepts OopHandle or int for lifetime release",
            },
            PyNativeCompatibilityMethod {
                python_method: "needs_commit",
                native_method: "NativeSession.needs_commit",
                native_return: "bool",
                python_return: "bool",
                note: "passes transaction state through unchanged",
            },
            PyNativeCompatibilityMethod {
                python_method: "in_transaction",
                native_method: "NativeSession.in_transaction",
                native_return: "bool",
                python_return: "bool",
                note: "passes transaction state through unchanged",
            },
            PyNativeCompatibilityMethod {
                python_method: "commit",
                native_method: "NativeSession.commit",
                native_return: "None",
                python_return: "None",
                note: "commits through the shared Rust core",
            },
            PyNativeCompatibilityMethod {
                python_method: "abort",
                native_method: "NativeSession.abort",
                native_return: "None",
                python_return: "None",
                note: "aborts through the shared Rust core",
            },
            PyNativeCompatibilityMethod {
                python_method: "logout",
                native_method: "NativeSession.logout",
                native_return: "None",
                python_return: "None",
                note: "logs out through the shared Rust core",
            },
        ],
    }
}

pub const PY_NATIVE_MODULE_FUNCTIONS: &[&str] = &[
    "capabilities_json",
    "samples_json",
    "smoke_dry_run_json",
    "migration_json",
    "compatibility_json",
    "conformance_json",
    "handoff_json",
];

pub const PY_NATIVE_SESSION_METHODS: &[&str] = &[
    "login_from_env",
    "session_id",
    "eval_repr",
    "eval_json",
    "eval_smallint",
    "eval_oop",
    "execute",
    "resolve",
    "value_to_oop_nil",
    "value_to_oop_bool",
    "value_to_oop_smallint",
    "value_to_oop_char",
    "value_to_oop_string",
    "value_to_oop_symbol",
    "value_to_oop_raw",
    "perform_raw_oop",
    "perform_json",
    "new_string",
    "new_symbol",
    "fetch_string",
    "global_get",
    "global_put_raw",
    "global_put_string",
    "global_put_smallint",
    "add_to_export_set",
    "remove_from_export_set",
    "needs_commit",
    "in_transaction",
    "commit",
    "abort",
    "logout",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyNativeConformanceFixture {
    pub path: &'static str,
    pub command: &'static str,
    pub purpose: &'static str,
}

impl PyNativeConformanceFixture {
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"path":"{}","command":"{}","purpose":"{}"}}"#,
            json_escape(self.path),
            json_escape(self.command),
            json_escape(self.purpose)
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyNativeScaffoldFile {
    pub path: &'static str,
    pub purpose: &'static str,
}

impl PyNativeScaffoldFile {
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"path":"{}","purpose":"{}"}}"#,
            json_escape(self.path),
            json_escape(self.purpose)
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyNativeConformanceReport {
    pub contract_version: u16,
    pub target_package: &'static str,
    pub status: &'static str,
    pub module_functions: &'static [&'static str],
    pub native_session_methods: &'static [&'static str],
    pub compatibility_methods: Vec<&'static str>,
    pub fixtures: Vec<PyNativeConformanceFixture>,
    pub scaffold_files: Vec<PyNativeScaffoldFile>,
}

impl PyNativeConformanceReport {
    pub fn to_json(&self) -> String {
        let fixtures = self
            .fixtures
            .iter()
            .map(PyNativeConformanceFixture::to_json)
            .collect::<Vec<_>>()
            .join(",");
        let scaffold_files = self
            .scaffold_files
            .iter()
            .map(PyNativeScaffoldFile::to_json)
            .collect::<Vec<_>>()
            .join(",");
        format!(
            r#"{{"contractVersion":{},"targetPackage":"{}","status":"{}","moduleFunctions":[{}],"nativeSessionMethods":[{}],"compatibilityMethods":[{}],"fixtures":[{}],"scaffoldFiles":[{}]}}"#,
            self.contract_version,
            json_escape(self.target_package),
            json_escape(self.status),
            json_string_array(self.module_functions),
            json_string_array(self.native_session_methods),
            json_string_array(&self.compatibility_methods),
            fixtures,
            scaffold_files
        )
    }
}

pub fn conformance_report() -> PyNativeConformanceReport {
    PyNativeConformanceReport {
        contract_version: capabilities().contract_version,
        target_package: "gemstone-py-native",
        status: "Generated PyO3 scaffold and downstream gemstone-py-native expose the Rust-backed native surface; local live smoke and published wheel verification have passed",
        module_functions: PY_NATIVE_MODULE_FUNCTIONS,
        native_session_methods: PY_NATIVE_SESSION_METHODS,
        compatibility_methods: compatibility_report()
            .methods
            .iter()
            .map(|method| method.python_method)
            .collect(),
        fixtures: vec![
            PyNativeConformanceFixture {
                path: "examples/py-native/gemstone-rs.py-native.json",
                command: "gemstone-rs py-native check examples/py-native/gemstone-rs.py-native.json",
                purpose: "Rust adapter capability contract",
            },
            PyNativeConformanceFixture {
                path: "examples/py-native/gemstone-rs.py-native-samples.json",
                command: "gemstone-rs py-native check-samples examples/py-native/gemstone-rs.py-native-samples.json",
                purpose: "Value and structured error translation samples",
            },
            PyNativeConformanceFixture {
                path: "examples/py-native/gemstone-rs.py-native-smoke.json",
                command: "gemstone-rs py-native check-smoke examples/py-native/gemstone-rs.py-native-smoke.json",
                purpose: "Dependency-free dry-run adapter smoke report",
            },
            PyNativeConformanceFixture {
                path: "examples/py-native/gemstone-rs.py-native-compat.json",
                command: "gemstone-rs py-native check-compat examples/py-native/gemstone-rs.py-native-compat.json",
                purpose: "Python compatibility shim method and return policy",
            },
            PyNativeConformanceFixture {
                path: "examples/py-native/gemstone-rs.py-native-conformance.json",
                command: "gemstone-rs py-native check-conformance examples/py-native/gemstone-rs.py-native-conformance.json",
                purpose: "End-to-end scaffold conformance target for wrapper integration",
            },
            PyNativeConformanceFixture {
                path: "examples/py-native/gemstone-rs.py-native-handoff.json",
                command: "gemstone-rs py-native check-handoff examples/py-native/gemstone-rs.py-native-handoff.json",
                purpose: "Downstream wrapper handoff manifest and acceptance criteria",
            },
        ],
        scaffold_files: vec![
            PyNativeScaffoldFile {
                path: "Cargo.toml",
                purpose: "Rust package metadata and gemstone-rs dependency",
            },
            PyNativeScaffoldFile {
                path: "pyproject.toml",
                purpose: "maturin Python extension metadata",
            },
            PyNativeScaffoldFile {
                path: "src/lib.rs",
                purpose: "PyO3 extension module and NativeSession wrapper",
            },
            PyNativeScaffoldFile {
                path: "src/main.rs",
                purpose: "Rust-side scaffold smoke executable",
            },
            PyNativeScaffoldFile {
                path: "PYTHON.md",
                purpose: "Python build and live smoke instructions",
            },
            PyNativeScaffoldFile {
                path: "python/gemstone_py_native_compat.py",
                purpose: "Python package-layer compatibility shim",
            },
            PyNativeScaffoldFile {
                path: "tests/test_smoke.py",
                purpose: "pytest smoke tests for module functions, session methods, and compatibility policy",
            },
        ],
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyNativeHandoffArtifact {
    pub name: &'static str,
    pub path: &'static str,
    pub schema: &'static str,
    pub command: &'static str,
    pub check_command: &'static str,
    pub purpose: &'static str,
}

impl PyNativeHandoffArtifact {
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"name":"{}","path":"{}","schema":"{}","command":"{}","checkCommand":"{}","purpose":"{}"}}"#,
            json_escape(self.name),
            json_escape(self.path),
            json_escape(self.schema),
            json_escape(self.command),
            json_escape(self.check_command),
            json_escape(self.purpose)
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyNativeAcceptanceCriterion {
    pub id: &'static str,
    pub required: bool,
    pub verify: &'static str,
}

impl PyNativeAcceptanceCriterion {
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"id":"{}","required":{},"verify":"{}"}}"#,
            json_escape(self.id),
            if self.required { "true" } else { "false" },
            json_escape(self.verify)
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyNativeHandoffReport {
    pub contract_version: u16,
    pub target_package: &'static str,
    pub adapter_module: &'static str,
    pub scaffold: &'static str,
    pub status: &'static str,
    pub artifacts: Vec<PyNativeHandoffArtifact>,
    pub acceptance: Vec<PyNativeAcceptanceCriterion>,
}

impl PyNativeHandoffReport {
    pub fn to_json(&self) -> String {
        let artifacts = self
            .artifacts
            .iter()
            .map(PyNativeHandoffArtifact::to_json)
            .collect::<Vec<_>>()
            .join(",");
        let acceptance = self
            .acceptance
            .iter()
            .map(PyNativeAcceptanceCriterion::to_json)
            .collect::<Vec<_>>()
            .join(",");
        format!(
            r#"{{"contractVersion":{},"targetPackage":"{}","adapterModule":"{}","scaffold":"{}","status":"{}","artifacts":[{}],"acceptance":[{}]}}"#,
            self.contract_version,
            json_escape(self.target_package),
            json_escape(self.adapter_module),
            json_escape(self.scaffold),
            json_escape(self.status),
            artifacts,
            acceptance
        )
    }
}

pub fn handoff_report() -> PyNativeHandoffReport {
    PyNativeHandoffReport {
        contract_version: capabilities().contract_version,
        target_package: "gemstone-py-native",
        adapter_module: "gemstone_rs::py_native",
        scaffold: "py_native_pyo3_adapter",
        status: "Rust-side handoff bundle is wired into gemstone-py-native; local live smoke and publish verification have passed",
        artifacts: vec![
            PyNativeHandoffArtifact {
                name: "capabilities",
                path: "examples/py-native/gemstone-rs.py-native.json",
                schema: "schemas/gemstone-rs.py-native.schema.json",
                command: "gemstone-rs py-native capabilities --json",
                check_command: "gemstone-rs py-native check examples/py-native/gemstone-rs.py-native.json",
                purpose: "Adapter operations, value kinds, error kinds, threading policy, and OOP constants",
            },
            PyNativeHandoffArtifact {
                name: "samples",
                path: "examples/py-native/gemstone-rs.py-native-samples.json",
                schema: "schemas/gemstone-rs.py-native-samples.schema.json",
                command: "gemstone-rs py-native samples --json",
                check_command: "gemstone-rs py-native check-samples examples/py-native/gemstone-rs.py-native-samples.json",
                purpose: "Value and structured error payloads for Python wrapper translation tests",
            },
            PyNativeHandoffArtifact {
                name: "smoke",
                path: "examples/py-native/gemstone-rs.py-native-smoke.json",
                schema: "schemas/gemstone-rs.py-native-smoke.schema.json",
                command: "gemstone-rs py-native smoke --dry-run --json",
                check_command: "gemstone-rs py-native check-smoke examples/py-native/gemstone-rs.py-native-smoke.json",
                purpose: "Dependency-free adapter smoke report for CI without a live stone",
            },
            PyNativeHandoffArtifact {
                name: "migration",
                path: "",
                schema: "schemas/gemstone-rs.py-native-migration.schema.json",
                command: "gemstone-rs py-native migration --json",
                check_command: "",
                purpose: "Downstream wrapper status plus live-smoke and wheel-publish checklist",
            },
            PyNativeHandoffArtifact {
                name: "compatibility",
                path: "examples/py-native/gemstone-rs.py-native-compat.json",
                schema: "schemas/gemstone-rs.py-native-compat.schema.json",
                command: "gemstone-rs py-native compatibility --json",
                check_command: "gemstone-rs py-native check-compat examples/py-native/gemstone-rs.py-native-compat.json",
                purpose: "Python package-layer return policy and method mapping",
            },
            PyNativeHandoffArtifact {
                name: "conformance",
                path: "examples/py-native/gemstone-rs.py-native-conformance.json",
                schema: "schemas/gemstone-rs.py-native-conformance.schema.json",
                command: "gemstone-rs py-native conformance --json",
                check_command: "gemstone-rs py-native check-conformance examples/py-native/gemstone-rs.py-native-conformance.json",
                purpose: "PyO3 module functions, raw session methods, shim methods, fixtures, and scaffold files",
            },
        ],
        acceptance: vec![
            PyNativeAcceptanceCriterion {
                id: "scaffold_compiles",
                required: true,
                verify: "python3 scripts/check_py_native_pyo3_scaffold.py",
            },
            PyNativeAcceptanceCriterion {
                id: "fixtures_current",
                required: true,
                verify: "cargo run -p gemstone-rs-cli -- py-native check-all",
            },
            PyNativeAcceptanceCriterion {
                id: "python_return_policy_preserved",
                required: true,
                verify: "gemstone-py sync and async test suites keep existing Session.execute()/perform() behavior",
            },
            PyNativeAcceptanceCriterion {
                id: "live_native_backend_green",
                required: true,
                verify: "GS_RUN_LIVE=1 gemstone-py native/backend smoke through the Rust-backed PyO3 module",
            },
            PyNativeAcceptanceCriterion {
                id: "wheels_after_live_green",
                required: true,
                verify: "gemstone-py-native 0.1.3 TestPyPI/PyPI install verification passed through the native-wheels workflow",
            },
        ],
    }
}

pub fn nil_oop() -> u64 {
    Oop::NIL.raw()
}

pub fn bool_oop(value: bool) -> u64 {
    Oop::from_bool(value).raw()
}

pub fn smallint_oop(value: i64) -> u64 {
    Oop::from_smallint(value).raw()
}

pub fn char_oop(value: char) -> u64 {
    Oop::from_char(value).raw()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyNativeSmokeStep {
    pub name: &'static str,
    pub ok: bool,
    pub detail: String,
}

impl PyNativeSmokeStep {
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"name":"{}","ok":{},"detail":"{}"}}"#,
            json_escape(self.name),
            if self.ok { "true" } else { "false" },
            json_escape(&self.detail)
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyNativeSmokeReport {
    pub dry_run: bool,
    pub contract_version: u16,
    pub steps: Vec<PyNativeSmokeStep>,
}

impl PyNativeSmokeReport {
    pub fn ok(&self) -> bool {
        self.steps.iter().all(|step| step.ok)
    }

    pub fn to_json(&self) -> String {
        let steps = self
            .steps
            .iter()
            .map(PyNativeSmokeStep::to_json)
            .collect::<Vec<_>>()
            .join(",");
        format!(
            r#"{{"ok":{},"dryRun":{},"contractVersion":{},"steps":[{}]}}"#,
            if self.ok() { "true" } else { "false" },
            if self.dry_run { "true" } else { "false" },
            self.contract_version,
            steps
        )
    }
}

pub fn smoke_dry_run_report() -> PyNativeSmokeReport {
    let mut steps = Vec::new();
    smoke_contract_steps(&mut steps);
    PyNativeSmokeReport {
        dry_run: true,
        contract_version: capabilities().contract_version,
        steps,
    }
}

pub fn smoke_live_report() -> PyNativeSmokeReport {
    let mut steps = Vec::new();
    smoke_contract_steps(&mut steps);
    smoke_live_steps(&mut steps);
    PyNativeSmokeReport {
        dry_run: false,
        contract_version: capabilities().contract_version,
        steps,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyNativeValueSample {
    pub name: &'static str,
    pub value: PyNativeValue,
}

impl PyNativeValueSample {
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"name":"{}","value":{}}}"#,
            json_escape(self.name),
            self.value.to_json()
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyNativeErrorSample {
    pub name: &'static str,
    pub error: PyNativeErrorInfo,
}

impl PyNativeErrorSample {
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"name":"{}","error":{}}}"#,
            json_escape(self.name),
            self.error.to_json()
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyNativeSamplesReport {
    pub contract_version: u16,
    pub values: Vec<PyNativeValueSample>,
    pub errors: Vec<PyNativeErrorSample>,
}

impl PyNativeSamplesReport {
    pub fn to_json(&self) -> String {
        let values = self
            .values
            .iter()
            .map(PyNativeValueSample::to_json)
            .collect::<Vec<_>>()
            .join(",");
        let errors = self
            .errors
            .iter()
            .map(PyNativeErrorSample::to_json)
            .collect::<Vec<_>>()
            .join(",");
        format!(
            r#"{{"contractVersion":{},"values":[{}],"errors":[{}]}}"#,
            self.contract_version, values, errors
        )
    }
}

pub fn samples_report() -> PyNativeSamplesReport {
    PyNativeSamplesReport {
        contract_version: capabilities().contract_version,
        values: vec![
            PyNativeValueSample {
                name: "nil",
                value: PyNativeValue::Nil,
            },
            PyNativeValueSample {
                name: "true",
                value: PyNativeValue::Bool(true),
            },
            PyNativeValueSample {
                name: "smallint",
                value: PyNativeValue::SmallInt(7),
            },
            PyNativeValueSample {
                name: "char",
                value: PyNativeValue::Char('A'),
            },
            PyNativeValueSample {
                name: "string",
                value: PyNativeValue::String("hello gemstone".to_string()),
            },
            PyNativeValueSample {
                name: "symbol",
                value: PyNativeValue::Symbol("printString".to_string()),
            },
            PyNativeValueSample {
                name: "oop",
                value: PyNativeValue::Oop(Oop::from_smallint(7).raw()),
            },
        ],
        errors: vec![
            PyNativeErrorSample {
                name: "missingConfig",
                error: PyNativeErrorInfo::from_error(&Error::MissingConfig("username")),
            },
            PyNativeErrorSample {
                name: "illegalOop",
                error: PyNativeErrorInfo::from_error(&Error::IllegalOop { operation: "eval" }),
            },
            PyNativeErrorSample {
                name: "unexpectedType",
                error: PyNativeErrorInfo::from_error(&Error::UnexpectedType {
                    expected: "SmallInt",
                    actual: "String(\"7\")".to_string(),
                }),
            },
            PyNativeErrorSample {
                name: "mapping",
                error: PyNativeErrorInfo::from_error(&Error::Mapping {
                    field: "booking.customer.name".to_string(),
                    expected: "String",
                    actual: "SmallInt(7)".to_string(),
                }),
            },
        ],
    }
}

fn smoke_contract_steps(steps: &mut Vec<PyNativeSmokeStep>) {
    let capabilities = capabilities();
    push_smoke_step(
        steps,
        "capabilities",
        capabilities.contract_version == 1
            && capabilities.operations.contains(&"eval")
            && capabilities.operations.contains(&"perform")
            && capabilities.operations.contains(&"global_put"),
        format!(
            "contract_version={} operations={}",
            capabilities.contract_version,
            capabilities.operations.len()
        ),
    );
    push_smoke_step(
        steps,
        "oop_constants",
        nil_oop() == Oop::NIL.raw()
            && bool_oop(true) == Oop::TRUE.raw()
            && bool_oop(false) == Oop::FALSE.raw()
            && smallint_oop(7) == Oop::from_smallint(7).raw()
            && char_oop('A') == Oop::from_char('A').raw(),
        "nil/true/false/smallint/char constants match gemstone-rs Oop helpers",
    );
    push_smoke_step(
        steps,
        "value_conversion",
        PyNativeValue::from(Value::SmallInt(7)) == PyNativeValue::SmallInt(7)
            && PyNativeValue::from(Value::Oop(Oop(1234))).raw_oop() == Some(1234)
            && PyNativeValue::Symbol("abc".to_string())
                .to_value()
                .is_none(),
        "plain Value <-> PyNativeValue conversion is stable",
    );
    let config_error = PyNativeConfig::default().into_config().unwrap_err();
    let config_info = PyNativeErrorInfo::from_error(&config_error);
    push_smoke_step(
        steps,
        "config_error_mapping",
        matches!(config_error, Error::MissingConfig("username"))
            && config_info.kind == PyNativeErrorKind::MissingConfig,
        config_error.to_string(),
    );
    let oop_error = Error::IllegalOop { operation: "eval" };
    let oop_info = PyNativeErrorInfo::from_error(&oop_error);
    push_smoke_step(
        steps,
        "structured_error_mapping",
        oop_info.kind == PyNativeErrorKind::IllegalOop && oop_info.operation == Some("eval"),
        oop_error.to_string(),
    );
}

fn smoke_live_steps(steps: &mut Vec<PyNativeSmokeStep>) {
    let config = match PyNativeConfig::from_env() {
        Ok(config) => {
            let summary = config.redacted_summary();
            push_smoke_step(
                steps,
                "config_from_env",
                true,
                format!(
                    "stone={} host={} user={} password_set={}",
                    summary.stone, summary.host, summary.username, summary.password_set
                ),
            );
            config
        }
        Err(err) => {
            push_smoke_step(steps, "config_from_env", false, err.to_string());
            return;
        }
    };

    let mut session = match PyNativeSession::login(config) {
        Ok(session) => {
            push_smoke_step(
                steps,
                "login",
                true,
                format!("session_id={}", session.session_id()),
            );
            session
        }
        Err(err) => {
            push_smoke_step(steps, "login", false, err.to_string());
            return;
        }
    };

    push_result_step(
        steps,
        "eval_3_plus_4",
        session
            .eval("3 + 4")
            .map(|value| (value == PyNativeValue::SmallInt(7), format!("{value:?}"))),
    );
    push_result_step(
        steps,
        "perform_print_string",
        session
            .perform_values(PyNativeValue::SmallInt(7), "printString", &[])
            .and_then(|printed| {
                let raw = printed.raw_oop().ok_or(Error::UnexpectedType {
                    expected: "Oop",
                    actual: format!("{printed:?}"),
                })?;
                session.fetch_string(raw)
            })
            .map(|value| (value == "7", value)),
    );

    let key = format!("GemStoneRsPyNative{}", std::process::id());
    let global_round_trip = session
        .global_put_value(&key, PyNativeValue::String("shared core".to_string()))
        .and_then(|_| session.commit())
        .and_then(|_| session.global_get(&key))
        .and_then(|oop| session.fetch_string(oop))
        .map(|value| (value == "shared core", value));
    push_result_step(steps, "global_string_round_trip", global_round_trip);

    let cleanup = session
        .global_put_raw(&key, nil_oop())
        .and_then(|_| session.commit())
        .map(|_| (true, format!("{key}=nil")));
    push_result_step(steps, "cleanup", cleanup);

    push_result_step(
        steps,
        "logout",
        session.logout().map(|_| (true, "logged out".to_string())),
    );
}

fn push_result_step(
    steps: &mut Vec<PyNativeSmokeStep>,
    name: &'static str,
    result: Result<(bool, String)>,
) {
    match result {
        Ok((ok, detail)) => push_smoke_step(steps, name, ok, detail),
        Err(err) => push_smoke_step(steps, name, false, err.to_string()),
    }
}

fn push_smoke_step(
    steps: &mut Vec<PyNativeSmokeStep>,
    name: &'static str,
    ok: bool,
    detail: impl Into<String>,
) {
    steps.push(PyNativeSmokeStep {
        name,
        ok,
        detail: detail.into(),
    });
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::new();
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

fn json_string_array(values: &[&str]) -> String {
    values
        .iter()
        .map(|value| format!(r#""{}""#, json_escape(value)))
        .collect::<Vec<_>>()
        .join(",")
}

pub struct PyNativeSession {
    session: Session,
}

impl PyNativeSession {
    pub fn login(config: PyNativeConfig) -> Result<Self> {
        Self::login_config(config.into_config()?)
    }

    pub fn login_config(config: Config) -> Result<Self> {
        Ok(Self {
            session: Session::login(config)?,
        })
    }

    pub fn login_from_env() -> Result<Self> {
        Self::login_config(Config::from_env()?)
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn session_mut(&mut self) -> &mut Session {
        &mut self.session
    }

    pub fn session_id(&self) -> i32 {
        self.session.session_id()
    }

    pub fn is_logged_in(&self) -> bool {
        self.session.is_logged_in()
    }

    pub fn eval(&mut self, source: &str) -> Result<PyNativeValue> {
        self.session.eval(source).map(PyNativeValue::from_value)
    }

    pub fn eval_oop(&mut self, source: &str) -> Result<u64> {
        self.session.eval_oop(source).map(Oop::raw)
    }

    pub fn execute(&mut self, source: &str) -> Result<u64> {
        self.session.execute(source).map(Oop::raw)
    }

    pub fn resolve(&mut self, name: &str) -> Result<u64> {
        self.session.resolve(name).map(Oop::raw)
    }

    pub fn value_to_oop(&mut self, value: PyNativeValue) -> Result<u64> {
        value.to_oop(&mut self.session).map(Oop::raw)
    }

    pub fn perform_raw(
        &mut self,
        receiver: u64,
        selector: &str,
        args: &[u64],
    ) -> Result<PyNativeValue> {
        let args = args.iter().copied().map(Oop).collect::<Vec<_>>();
        self.session
            .perform(Oop(receiver), selector, &args)
            .map(PyNativeValue::from_value)
    }

    pub fn perform_values(
        &mut self,
        receiver: PyNativeValue,
        selector: &str,
        args: &[PyNativeValue],
    ) -> Result<PyNativeValue> {
        let receiver = receiver.to_oop(&mut self.session)?;
        let args = args
            .iter()
            .map(|arg| arg.to_oop(&mut self.session))
            .collect::<Result<Vec<_>>>()?;
        self.session
            .perform(receiver, selector, &args)
            .map(PyNativeValue::from_value)
    }

    pub fn perform_oop_raw(&mut self, receiver: u64, selector: &str, args: &[u64]) -> Result<u64> {
        let args = args.iter().copied().map(Oop).collect::<Vec<_>>();
        self.session
            .perform_oop(Oop(receiver), selector, &args)
            .map(Oop::raw)
    }

    pub fn new_string(&mut self, value: &str) -> Result<u64> {
        self.session.new_string(value).map(Oop::raw)
    }

    pub fn new_symbol(&mut self, value: &str) -> Result<u64> {
        self.session.new_symbol(value).map(Oop::raw)
    }

    pub fn fetch_string(&mut self, oop: u64) -> Result<String> {
        self.session.fetch_string(Oop(oop))
    }

    pub fn global_get(&mut self, symbol_name: &str) -> Result<u64> {
        self.session.global_get(symbol_name).map(Oop::raw)
    }

    pub fn global_put_raw(&mut self, symbol_name: &str, value: u64) -> Result<()> {
        self.session.global_put(symbol_name, Oop(value))
    }

    pub fn global_put_value(&mut self, symbol_name: &str, value: PyNativeValue) -> Result<()> {
        let value = value.to_oop(&mut self.session)?;
        self.session.global_put(symbol_name, value)
    }

    pub fn add_to_export_set(&mut self, oop: u64) -> Result<()> {
        self.session.add_to_export_set(Oop(oop))
    }

    pub fn remove_from_export_set(&mut self, oop: u64) -> Result<()> {
        self.session.remove_from_export_set(Oop(oop))
    }

    pub fn needs_commit(&mut self) -> Result<bool> {
        self.session.needs_commit()
    }

    pub fn in_transaction(&mut self) -> Result<bool> {
        self.session.in_transaction()
    }

    pub fn commit(&mut self) -> Result<()> {
        self.session.commit()
    }

    pub fn abort(&mut self) -> Result<()> {
        self.session.abort()
    }

    pub fn logout(&mut self) -> Result<()> {
        self.session.logout()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_round_trips_without_leaking_passwords() -> Result<()> {
        let config = Config::builder()
            .stone("seaside")
            .netldi("netldi")
            .host("db.example.test")
            .username("DataCurator")
            .password("secret")
            .host_username("hostuser")
            .host_password("hostsecret")
            .gem_service("gemnetobject")
            .lib_path("/opt/gemstone/lib/libgcirpc.dylib")
            .build()?;

        let py_config = PyNativeConfig::from_config(config.clone());
        let summary = py_config.redacted_summary();
        assert_eq!(summary.stone, "seaside");
        assert_eq!(summary.username, "DataCurator");
        assert!(summary.password_set);
        assert!(summary.host_password_set);
        assert_eq!(py_config.into_config()?, config);
        Ok(())
    }

    #[test]
    fn config_conversion_preserves_required_field_errors() {
        let err = PyNativeConfig::default().into_config().unwrap_err();
        assert!(matches!(err, Error::MissingConfig("username")));
    }

    #[test]
    fn value_conversion_is_plain_and_explicit() {
        assert_eq!(
            PyNativeValue::from(Value::SmallInt(7)),
            PyNativeValue::SmallInt(7)
        );
        assert_eq!(
            PyNativeValue::from(Value::Oop(Oop(1234))).raw_oop(),
            Some(1234)
        );
        assert_eq!(PyNativeValue::Symbol("abc".to_string()).to_value(), None);
        assert_eq!(PyNativeValue::Oop(20).to_value(), Some(Value::Oop(Oop(20))));
        assert_eq!(PyNativeValue::String("abc".to_string()).kind(), "string");
        assert_eq!(
            PyNativeValue::Symbol("abc".to_string()).to_json(),
            r#"{"kind":"symbol","value":"abc"}"#
        );
        assert_eq!(nil_oop(), Oop::NIL.raw());
        assert_eq!(bool_oop(true), Oop::TRUE.raw());
        assert_eq!(smallint_oop(7), Oop::from_smallint(7).raw());
        assert_eq!(char_oop('A'), Oop::from_char('A').raw());
    }

    #[test]
    fn error_info_preserves_structured_context() {
        let err = Error::GemStone {
            number: 2406,
            fatal: false,
            message: "compile error".to_string(),
        };
        let info = PyNativeErrorInfo::from_error(&err);
        assert_eq!(info.kind, PyNativeErrorKind::GemStone);
        assert_eq!(info.gemstone_number, Some(2406));
        assert_eq!(info.fatal, Some(false));

        let err = Error::IllegalOop { operation: "eval" };
        let info = PyNativeErrorInfo::from_error(&err);
        assert_eq!(info.kind, PyNativeErrorKind::IllegalOop);
        assert_eq!(info.operation, Some("eval"));
        assert_eq!(info.kind.as_json_name(), "illegalOop");
        assert!(info.to_json().contains(r#""operation":"eval""#));
    }

    #[test]
    fn capability_report_names_the_pyo3_contract() {
        let report = capabilities();
        assert_eq!(report.contract_version, 1);
        assert!(report.threading.contains("non-Send/non-Sync"));
        assert!(report.operations.contains(&"eval"));
        assert!(report.operations.contains(&"perform"));
        assert!(report.operations.contains(&"add_to_export_set"));
        assert_eq!(PY_NATIVE_VALUE_KINDS[0], "nil");
        assert_eq!(PY_NATIVE_ERROR_KINDS[0], "gci");
    }

    #[test]
    fn capability_report_json_is_stable_and_shared() {
        let json = capabilities().to_json();
        assert!(json.contains(r#""name":"gemstone-py-native adapter contract""#));
        assert!(json.contains(r#""contractVersion":1"#));
        assert!(json.contains(r#""operations":["login","logout","eval""#));
        assert!(json.contains(r#""valueKinds":["nil","bool","smallInt""#));
        assert!(json.contains(r#""errorKinds":["gci","missingEnvironment""#));
        assert!(json.contains(r#""oopConstants":"#));
    }

    #[test]
    fn smoke_report_json_is_stable_and_shared() {
        let report = smoke_dry_run_report();
        let json = report.to_json();
        assert!(json.contains(r#""ok":true"#));
        assert!(json.contains(r#""dryRun":true"#));
        assert!(json.contains(r#""contractVersion":1"#));
        assert!(json.contains(r#""name":"value_conversion""#));
        assert!(json.contains(r#""detail":"plain Value <-> PyNativeValue conversion is stable""#));
    }

    #[test]
    fn smoke_step_json_escapes_details() {
        let step = PyNativeSmokeStep {
            name: "quoted",
            ok: false,
            detail: "a\"b\\c\n".to_string(),
        };
        assert_eq!(
            step.to_json(),
            r#"{"name":"quoted","ok":false,"detail":"a\"b\\c\n"}"#
        );
    }

    #[test]
    fn samples_report_covers_value_and_error_shapes() {
        let report = samples_report();
        let json = report.to_json();
        assert_eq!(report.contract_version, 1);
        assert_eq!(report.values.len(), PY_NATIVE_VALUE_KINDS.len());
        assert!(json.contains(r#""name":"symbol","value":{"kind":"symbol","value":"printString"}"#));
        assert!(json.contains(r#""name":"mapping","error":{"kind":"mapping""#));
        assert!(json.contains(r#""field":"booking.customer.name""#));
    }

    #[test]
    fn migration_report_tracks_python_wrapper_work() {
        let report = migration_report();
        let json = report.to_json();
        assert_eq!(report.contract_version, 1);
        assert_eq!(report.target_package, "gemstone-py-native");
        assert_eq!(report.done_count(), 5);
        assert_eq!(report.pending_count(), 0);
        assert!(json.contains(r#""id":"wrap_py_native_session""#));
        assert!(!json.contains(r#""status":"pending""#));
        assert!(json.contains(r#""pendingCount":0"#));
        assert!(json.contains("run_native_rust_core_live_smoke.py"));
        assert!(json.contains("RustCoreSession"));
    }

    #[test]
    fn compatibility_report_documents_python_return_policy() {
        let report = compatibility_report();
        let json = report.to_json();
        assert_eq!(report.contract_version, 1);
        assert_eq!(report.module, "gemstone_py_native_compat");
        assert_eq!(report.session_class, "NativeCompatibilitySession");
        assert_eq!(report.handle_class, "OopHandle");
        assert!(report.return_policy.contains("typed helpers are opt-in"));
        assert!(report.methods.iter().any(|method| {
            method.python_method == "eval_oop"
                && method.native_method == "NativeSession.eval_oop"
                && method.python_return == "OopHandle"
        }));
        assert!(report.methods.iter().any(|method| {
            method.python_method == "eval_smallint" && method.python_return == "int"
        }));
        assert!(report.methods.iter().any(|method| {
            method.python_method == "eval_value"
                && method.native_method == "NativeSession.eval_json"
                && method.python_return == "dict"
        }));
        assert!(json.contains(r#""pythonMethod":"perform_oop""#));
        assert!(json.contains(r#""pythonMethod":"perform_value""#));
        assert!(json.contains(r#""nativeMethod":"NativeSession.perform_raw_oop""#));
        assert!(json.contains(r#""nativeMethod":"NativeSession.value_to_oop_symbol""#));
    }

    #[test]
    fn conformance_report_lists_scaffold_contract_surface() {
        let report = conformance_report();
        let json = report.to_json();
        assert_eq!(report.contract_version, 1);
        assert_eq!(report.target_package, "gemstone-py-native");
        assert!(report.module_functions.contains(&"compatibility_json"));
        assert!(report.module_functions.contains(&"conformance_json"));
        assert!(report.module_functions.contains(&"handoff_json"));
        assert!(report.native_session_methods.contains(&"eval_json"));
        assert!(report.native_session_methods.contains(&"perform_raw_oop"));
        assert!(report.native_session_methods.contains(&"perform_json"));
        assert!(report
            .native_session_methods
            .contains(&"value_to_oop_symbol"));
        assert!(report.compatibility_methods.contains(&"perform_oop"));
        assert!(report.compatibility_methods.contains(&"perform_value"));
        assert!(report
            .fixtures
            .iter()
            .any(|fixture| fixture.path.ends_with("gemstone-rs.py-native-compat.json")));
        assert!(report
            .fixtures
            .iter()
            .any(|fixture| fixture.path.ends_with("gemstone-rs.py-native-handoff.json")));
        assert!(report
            .scaffold_files
            .iter()
            .any(|file| file.path == "python/gemstone_py_native_compat.py"));
        assert!(json.contains(r#""targetPackage":"gemstone-py-native""#));
        assert!(json.contains(r#""moduleFunctions":["capabilities_json""#));
        assert!(json.contains(r#""nativeSessionMethods":["login_from_env""#));
    }

    #[test]
    fn handoff_report_lists_downstream_wrapper_artifacts() {
        let report = handoff_report();
        let json = report.to_json();
        assert_eq!(report.contract_version, 1);
        assert_eq!(report.target_package, "gemstone-py-native");
        assert_eq!(report.adapter_module, "gemstone_rs::py_native");
        assert!(report
            .artifacts
            .iter()
            .any(|artifact| artifact.name == "conformance"
                && artifact.check_command.contains("check-conformance")));
        assert!(report
            .artifacts
            .iter()
            .any(|artifact| artifact.name == "migration" && artifact.path.is_empty()));
        assert!(report
            .acceptance
            .iter()
            .any(|criterion| criterion.id == "live_native_backend_green" && criterion.required));
        assert!(json.contains(r#""targetPackage":"gemstone-py-native""#));
        assert!(json.contains(r#""adapterModule":"gemstone_rs::py_native""#));
        assert!(json.contains(r#""name":"conformance""#));
        assert!(json.contains(r#""id":"fixtures_current""#));
    }
}
