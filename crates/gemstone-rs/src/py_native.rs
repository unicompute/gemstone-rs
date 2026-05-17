//! Stable adapter surface for a future `gemstone-py-native` PyO3 wrapper.
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

#[derive(Clone, Debug, PartialEq)]
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PyNativeCapabilities {
    pub contract_version: u16,
    pub threading: &'static str,
    pub operations: &'static [&'static str],
}

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

pub fn capabilities() -> PyNativeCapabilities {
    PyNativeCapabilities {
        contract_version: 1,
        threading: "session is synchronous and non-Send/non-Sync; PyO3 wrappers should use unsendable classes or dedicated worker threads",
        operations: PY_NATIVE_OPERATIONS,
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
    }

    #[test]
    fn capability_report_names_the_pyo3_contract() {
        let report = capabilities();
        assert_eq!(report.contract_version, 1);
        assert!(report.threading.contains("non-Send/non-Sync"));
        assert!(report.operations.contains(&"eval"));
        assert!(report.operations.contains(&"perform"));
        assert!(report.operations.contains(&"add_to_export_set"));
    }
}
