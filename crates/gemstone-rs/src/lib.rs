//! Safe Rust client API for GemStone/S over GCI.
//!
//! This crate builds on `gemstone-gci`, which owns the dynamic `libgcirpc`
//! loading and raw ABI calls. `Session` provides RAII login/logout behavior,
//! explicit OOP values, conservative transaction helpers, and basic value
//! marshalling for nil, booleans, small integers, characters, and raw OOPs.
//!
//! Rust code imports this crate as `gemstone_rs`:
//!
//! ```no_run
//! use gemstone_rs::{Config, Session, Value};
//!
//! fn main() -> gemstone_rs::Result<()> {
//!     let config = Config::from_env()?;
//!     let mut session = Session::login(config)?;
//!
//!     let value = session.eval("3 + 4")?;
//!     assert_eq!(value, Value::SmallInt(7));
//!
//!     session.logout()?;
//!     Ok(())
//! }
//! ```
//!
//! Build configuration explicitly when you do not want to read process
//! environment:
//!
//! ```no_run
//! use gemstone_rs::{Config, Session};
//!
//! fn main() -> gemstone_rs::Result<()> {
//!     let config = Config::builder()
//!         .stone("gs64stone")
//!         .host("localhost")
//!         .netldi("netldi")
//!         .username("DataCurator")
//!         .password("your-password")
//!         .build()?;
//!     let mut session = Session::login(config)?;
//!     println!("session id: {}", session.session_id());
//!     Ok(())
//! }
//! ```
//!
//! Store and fetch values through `UserGlobals`:
//!
//! ```no_run
//! use gemstone_rs::{Config, Session};
//!
//! fn main() -> gemstone_rs::Result<()> {
//!     let mut session = Session::login(Config::from_env()?)?;
//!     let text = session.new_string("hello from Rust")?;
//!     session.global_put("GemStoneRsDocExample", text)?;
//!     session.commit()?;
//!
//!     let stored = session.global_get("GemStoneRsDocExample")?;
//!     assert_eq!(session.fetch_string(stored)?, "hello from Rust");
//!     Ok(())
//! }
//! ```
//!
//! Store a mapped Rust payload under the default bridge-root dictionary:
//!
//! ```no_run
//! use gemstone_rs::{BridgeValue, Config, Session};
//!
//! fn main() -> gemstone_rs::Result<()> {
//!     let mut session = Session::login(Config::from_env()?)?;
//!     let payload = BridgeValue::dictionary([
//!         ("name".to_string(), BridgeValue::from("Tariq")),
//!         ("amount".to_string(), BridgeValue::from(100_i64)),
//!         ("currency".to_string(), BridgeValue::from("GBP")),
//!     ]);
//!     let mut bridge_root = session.bridge_root()?;
//!     bridge_root.put("MyTestDict", payload)?;
//!     bridge_root.commit()?;
//!     Ok(())
//! }
//! ```
//!
//! Add a typed manual mapping on top of `BridgeValue` when you want Rust
//! structs at the application boundary:
//!
//! ```no_run
//! use gemstone_rs::{
//!     BridgeDictionary, BridgeFieldRead, BridgeFieldWrite, BridgeKeyType, BridgeMapped,
//!     BridgeValue, Config, Session,
//! };
//!
//! struct BookingDraft {
//!     name: String,
//!     amount: i64,
//!     note: Option<String>,
//! }
//!
//! impl BridgeMapped for BookingDraft {
//!     fn to_bridge_value(&self) -> BridgeValue {
//!         BridgeValue::dictionary([
//!             ("name".to_string(), BridgeValue::from(self.name.clone())),
//!             ("amount".to_string(), BridgeValue::from(self.amount)),
//!             ("note".to_string(), BridgeFieldWrite::to_bridge_field_value(&self.note)),
//!         ])
//!     }
//!
//!     fn from_bridge_dictionary(dictionary: &mut BridgeDictionary<'_>) -> gemstone_rs::Result<Self> {
//!         Ok(Self {
//!             name: dictionary.at_string("name")?,
//!             amount: dictionary.at_smallint("amount")?,
//!             note: BridgeFieldRead::read_bridge_field(
//!                 dictionary,
//!                 "note",
//!                 BridgeKeyType::String,
//!             )?,
//!         })
//!     }
//! }
//!
//! fn main() -> gemstone_rs::Result<()> {
//!     let mut session = Session::login(Config::from_env()?)?;
//!     let mut bridge_root = session.bridge_root()?;
//!     let draft = BookingDraft { name: "Tariq".to_string(), amount: 100, note: None };
//!     bridge_root.put_mapped("BookingDraft", &draft)?;
//!     let loaded: BookingDraft = bridge_root.get_mapped("BookingDraft")?;
//!     assert_eq!(loaded.amount, 100);
//!     Ok(())
//! }
//! ```
//!
//! Browse dictionaries, classes, protocols, methods, and source through the
//! same browser API used by the CLI and explorer:
//!
//! ```no_run
//! use gemstone_rs::{browser::Browser, Config, Session};
//!
//! fn main() -> gemstone_rs::Result<()> {
//!     let mut session = Session::login(Config::from_env()?)?;
//!     let mut browser = Browser::new(&mut session);
//!
//!     let dictionaries = browser.dictionaries()?;
//!     let classes = browser.classes("UserGlobals")?;
//!     let protocols = browser.protocols("Object", false, "")?;
//!     let methods = browser.methods("Object", "-- all --", false, "")?;
//!     let source = browser.source("Object", "printString", false, "")?;
//!
//!     println!("{dictionaries:?} {classes:?} {protocols:?} {methods:?}");
//!     println!("{source}");
//!     Ok(())
//! }
//! ```
//!
//! Call selectors directly when generated wrappers would be too much:
//!
//! ```no_run
//! use gemstone_rs::{Config, Session};
//!
//! fn main() -> gemstone_rs::Result<()> {
//!     let mut session = Session::login(Config::from_env()?)?;
//!     let seven = session.smallint_oop(7);
//!     let printed = session.perform_oop(seven, "printString", &[])?;
//!     assert_eq!(session.fetch_string(printed)?, "7");
//!     Ok(())
//! }
//! ```
//!
//! Transactions are explicit. `transaction` commits on success and aborts on
//! error:
//!
//! ```no_run
//! use gemstone_rs::{Config, Session};
//!
//! fn main() -> gemstone_rs::Result<()> {
//!     let mut session = Session::login(Config::from_env()?)?;
//!     session.transaction(|session| {
//!         let value = session.new_string("hello from Rust")?;
//!         session.global_put("GemStoneRsExample", value)
//!     })?;
//!     Ok(())
//! }
//! ```
//!
//! OOPs and values are explicit:
//!
//! ```no_run
//! use gemstone_rs::{Config, Session, Value};
//!
//! fn main() -> gemstone_rs::Result<()> {
//!     let mut session = Session::login(Config::from_env()?)?;
//!     let oop = session.value_to_oop(&Value::SmallInt(7))?;
//!     let value = session.perform(oop, "printString", &[])?;
//!     println!("{value:?}");
//!     Ok(())
//! }
//! ```
//!
//! Codegen configs can be parsed and checked from Rust:
//!
//! ```
//! use gemstone_rs::codegen;
//!
//! let config = codegen::Config::parse(
//!     "output = generated/gemstone_wrappers.rs\nclass = Object\nmethod = Object>>printString | return=String\n",
//!     None,
//! )?;
//! let generated = codegen::generate(&config);
//! assert!(generated.source.contains("pub struct Object"));
//! # Ok::<(), codegen::Error>(())
//! ```
//!
//! Check generated wrappers in build or release tooling:
//!
//! ```no_run
//! use gemstone_rs::codegen;
//!
//! fn main() -> codegen::Result<()> {
//!     let config = codegen::Config::from_file("examples/codegen/gemstone-rs.codegen")?;
//!     let report = codegen::check(&config)?;
//!     assert!(report.up_to_date, "{} is stale", report.output.display());
//!     Ok(())
//! }
//! ```
//!
//! Safety notes:
//!
//! - `Session` is deliberately not `Send` or `Sync`; keep it on the thread
//!   that logged in.
//! - Unsafe C ABI calls are isolated in `gemstone-gci`.
//! - `libgcirpc` is loaded dynamically at runtime through `GS_LIB_PATH`,
//!   `GS_LIB`, or `GEMSTONE/lib`.
//!
//! Roadmap:
//!
//! - Broader typed wrapper generation for Rust services and CLIs.
//! - A richer local explorer over the same API.
//! - Optional VS Code webview integration once the CLI and explorer contracts
//!   are stable.

extern crate self as gemstone_rs;

pub mod bridge;
pub mod browser;
pub mod codegen;
pub mod profiles;

pub use bridge::{
    BridgeDictionary, BridgeFieldRead, BridgeFieldWrite, BridgeKey, BridgeKeySummary,
    BridgeKeyType, BridgeMapped, BridgeRoot, BridgeValue, DEFAULT_BRIDGE_ROOT,
};
pub use gemstone_gci::Oop;
use gemstone_gci::{
    char_from_oop, is_char, is_smallint, resolve_library_path_with_source, GciErrSType, GciLibrary,
    RawOop, GCI_ENCRYPT_BUF_SIZE, GCI_INVALID_SESSION,
};
pub use gemstone_rs_macros::BridgeMapped;
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::env;
use std::error::Error as StdError;
use std::ffi::{c_char, c_double, c_uint, CString, NulError};
use std::fmt;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::rc::Rc;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Gci(gemstone_gci::GciError),
    MissingEnvironment(&'static str),
    MissingConfig(&'static str),
    Nul(NulError),
    NotLoggedIn,
    GemStone {
        number: i32,
        fatal: bool,
        message: String,
    },
    IllegalOop {
        operation: &'static str,
    },
    UnexpectedType {
        expected: &'static str,
        actual: String,
    },
    Mapping {
        field: String,
        expected: &'static str,
        actual: String,
    },
    NegativeSize(i64),
    ArgumentCountTooLarge(usize),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Gci(err) => write!(f, "{err}"),
            Self::MissingEnvironment(name) => {
                write!(f, "missing required GemStone environment variable {name}")
            }
            Self::MissingConfig(name) => write!(f, "missing required GemStone config field {name}"),
            Self::Nul(err) => write!(f, "string contains NUL byte: {err}"),
            Self::NotLoggedIn => write!(f, "GemStone session is not logged in"),
            Self::GemStone {
                number,
                fatal,
                message,
            } => write!(f, "GemStone error #{number} fatal={fatal}: {message}"),
            Self::IllegalOop { operation } => {
                write!(f, "GemStone operation {operation} returned OOP_ILLEGAL")
            }
            Self::UnexpectedType { expected, actual } => {
                write!(f, "expected GemStone value type {expected}, got {actual}")
            }
            Self::Mapping {
                field,
                expected,
                actual,
            } => write!(
                f,
                "field {field} expected GemStone value type {expected}, got {actual}"
            ),
            Self::NegativeSize(size) => write!(f, "GemStone returned a negative size: {size}"),
            Self::ArgumentCountTooLarge(count) => {
                write!(f, "too many GemStone selector arguments: {count}")
            }
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Gci(err) => Some(err),
            Self::Nul(err) => Some(err),
            Self::MissingEnvironment(_)
            | Self::MissingConfig(_)
            | Self::NotLoggedIn
            | Self::GemStone { .. }
            | Self::IllegalOop { .. }
            | Self::UnexpectedType { .. }
            | Self::Mapping { .. }
            | Self::NegativeSize(_)
            | Self::ArgumentCountTooLarge(_) => None,
        }
    }
}

impl From<gemstone_gci::GciError> for Error {
    fn from(value: gemstone_gci::GciError) -> Self {
        Self::Gci(value)
    }
}

impl From<NulError> for Error {
    fn from(value: NulError) -> Self {
        Self::Nul(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
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

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    pub fn from_env() -> Result<Self> {
        Self::builder()
            .stone(
                env::var("GS_STONE")
                    .or_else(|_| env::var("GS_STONE_NAME"))
                    .unwrap_or_else(|_| "gs64stone".to_string()),
            )
            .netldi(env::var("GS_NETLDI").unwrap_or_else(|_| "netldi".to_string()))
            .host(env::var("GS_HOST").unwrap_or_else(|_| "localhost".to_string()))
            .username(required_env("GS_USERNAME")?)
            .password(required_env("GS_PASSWORD")?)
            .host_username(env::var("GS_HOST_USERNAME").unwrap_or_default())
            .host_password(env::var("GS_HOST_PASSWORD").unwrap_or_default())
            .gem_service(env::var("GS_GEM_SERVICE").unwrap_or_else(|_| "gemnetobject".to_string()))
            .maybe_lib_path(
                env::var("GS_LIB_PATH")
                    .ok()
                    .filter(|value| !value.is_empty()),
            )
            .build()
    }

    pub fn stone_nrs(&self) -> String {
        if !self.host.is_empty() && self.host != "localhost" && self.host != "127.0.0.1" {
            format!("!@{}!{}!{}", self.host, self.netldi, self.stone)
        } else {
            self.stone.clone()
        }
    }
}

pub fn gci_library_path(config: &Config) -> Result<PathBuf> {
    Ok(GciLibrary::load(config.lib_path.clone())?
        .path()
        .to_path_buf())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GciLibraryResolution {
    pub source: &'static str,
    pub path: PathBuf,
    pub searched: Vec<PathBuf>,
}

pub fn gci_library_resolution(config: &Config) -> Result<GciLibraryResolution> {
    if let Some(path) = &config.lib_path {
        if env::var("GS_LIB_PATH")
            .ok()
            .filter(|value| !value.is_empty())
            .is_some_and(|value| PathBuf::from(value) == *path)
        {
            return Ok(GciLibraryResolution {
                source: "GS_LIB_PATH",
                path: path.clone(),
                searched: vec![path.clone()],
            });
        }
        return Ok(GciLibraryResolution {
            source: "config.lib_path",
            path: path.clone(),
            searched: vec![path.clone()],
        });
    }
    if let Ok(path) = env::var("GS_LIB_PATH") {
        if !path.is_empty() {
            let path = PathBuf::from(path);
            return Ok(GciLibraryResolution {
                source: "GS_LIB_PATH",
                path: path.clone(),
                searched: vec![path],
            });
        }
    }
    let resolution = resolve_library_path_with_source(None)?;
    Ok(GciLibraryResolution {
        source: resolution.source,
        path: resolution.path,
        searched: resolution.searched,
    })
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConfigBuilder {
    stone: Option<String>,
    netldi: Option<String>,
    host: Option<String>,
    username: Option<String>,
    password: Option<String>,
    host_username: Option<String>,
    host_password: Option<String>,
    gem_service: Option<String>,
    lib_path: Option<PathBuf>,
}

impl ConfigBuilder {
    pub fn stone(mut self, value: impl Into<String>) -> Self {
        self.stone = Some(value.into());
        self
    }

    pub fn netldi(mut self, value: impl Into<String>) -> Self {
        self.netldi = Some(value.into());
        self
    }

    pub fn host(mut self, value: impl Into<String>) -> Self {
        self.host = Some(value.into());
        self
    }

    pub fn username(mut self, value: impl Into<String>) -> Self {
        self.username = Some(value.into());
        self
    }

    pub fn password(mut self, value: impl Into<String>) -> Self {
        self.password = Some(value.into());
        self
    }

    pub fn host_username(mut self, value: impl Into<String>) -> Self {
        self.host_username = Some(value.into());
        self
    }

    pub fn host_password(mut self, value: impl Into<String>) -> Self {
        self.host_password = Some(value.into());
        self
    }

    pub fn gem_service(mut self, value: impl Into<String>) -> Self {
        self.gem_service = Some(value.into());
        self
    }

    pub fn lib_path(mut self, value: impl Into<PathBuf>) -> Self {
        self.lib_path = Some(value.into());
        self
    }

    pub fn maybe_lib_path(mut self, value: Option<impl Into<PathBuf>>) -> Self {
        self.lib_path = value.map(Into::into);
        self
    }

    pub fn build(self) -> Result<Config> {
        let username = required_config("username", self.username)?;
        let password = required_config("password", self.password)?;

        Ok(Config {
            stone: self.stone.unwrap_or_else(|| "gs64stone".to_string()),
            netldi: self.netldi.unwrap_or_else(|| "netldi".to_string()),
            host: self.host.unwrap_or_else(|| "localhost".to_string()),
            username,
            password,
            host_username: self.host_username.unwrap_or_default(),
            host_password: self.host_password.unwrap_or_default(),
            gem_service: self
                .gem_service
                .unwrap_or_else(|| "gemnetobject".to_string()),
            lib_path: self.lib_path,
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            stone: "gs64stone".to_string(),
            netldi: "netldi".to_string(),
            host: "localhost".to_string(),
            username: String::new(),
            password: String::new(),
            host_username: String::new(),
            host_password: String::new(),
            gem_service: "gemnetobject".to_string(),
            lib_path: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransactionPolicy {
    Manual,
    CommitOnSuccess,
    AbortOnExit,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    SmallInt(i64),
    Char(char),
    String(String),
    Oop(Oop),
}

impl Value {
    pub fn as_oop(&self) -> Option<Oop> {
        match self {
            Self::Oop(oop) => Some(*oop),
            _ => None,
        }
    }
}

pub struct Session {
    lib: GciLibrary,
    session_id: i32,
    logged_in: Cell<bool>,
    identity_map: RefCell<BTreeMap<RawOop, usize>>,
    next_identity: Cell<usize>,
    _not_send_sync: PhantomData<Rc<()>>,
}

impl Session {
    pub fn login(config: Config) -> Result<Self> {
        if config.username.is_empty() {
            return Err(Error::MissingEnvironment("GS_USERNAME"));
        }
        if config.password.is_empty() {
            return Err(Error::MissingEnvironment("GS_PASSWORD"));
        }

        let lib = GciLibrary::load(config.lib_path.clone())?;
        let mut session = Self {
            lib,
            session_id: GCI_INVALID_SESSION as i32,
            logged_in: Cell::new(false),
            identity_map: RefCell::new(BTreeMap::new()),
            next_identity: Cell::new(1),
            _not_send_sync: PhantomData,
        };
        session.open(config)?;
        Ok(session)
    }

    pub fn session_id(&self) -> i32 {
        self.session_id
    }

    pub fn is_logged_in(&self) -> bool {
        self.logged_in.get()
    }

    pub fn eval(&mut self, source: &str) -> Result<Value> {
        let oop = self.eval_oop(source)?;
        self.marshal(oop)
    }

    pub fn execute(&mut self, source: &str) -> Result<Oop> {
        self.eval_oop(source)
    }

    pub fn eval_oop(&mut self, source: &str) -> Result<Oop> {
        self.activate()?;
        let source = CString::new(source)?;
        let oop = unsafe { self.lib.gci_execute_str(&source, OOP_NIL)? };
        self.check_oop("eval", oop)?;
        Ok(Oop(oop))
    }

    pub fn perform(&mut self, receiver: Oop, selector: &str, args: &[Oop]) -> Result<Value> {
        let oop = self.perform_oop(receiver, selector, args)?;
        self.marshal(oop)
    }

    pub fn perform_oop(&mut self, receiver: Oop, selector: &str, args: &[Oop]) -> Result<Oop> {
        self.activate()?;
        let selector = CString::new(selector)?;
        if args.len() > i32::MAX as usize {
            return Err(Error::ArgumentCountTooLarge(args.len()));
        }
        let raw_args: Vec<RawOop> = args.iter().map(|oop| oop.raw()).collect();
        let oop = unsafe {
            self.lib.gci_perform(
                receiver.raw(),
                &selector,
                raw_args.as_ptr(),
                raw_args.len() as i32,
            )?
        };
        self.check_oop("perform", oop)?;
        Ok(Oop(oop))
    }

    pub fn resolve(&mut self, name: &str) -> Result<Oop> {
        self.activate()?;
        let name = CString::new(name)?;
        let oop = unsafe { self.lib.gci_resolve_symbol(&name, OOP_NIL)? };
        self.check_oop("resolve", oop)?;
        Ok(Oop(oop))
    }

    pub fn new_string(&mut self, value: &str) -> Result<Oop> {
        self.activate()?;
        let value = CString::new(value)?;
        let oop = unsafe { self.lib.gci_new_string(&value)? };
        self.check_oop("new_string", oop)?;
        Ok(Oop(oop))
    }

    pub fn new_symbol(&mut self, value: &str) -> Result<Oop> {
        self.activate()?;
        let value = CString::new(value)?;
        let oop = unsafe { self.lib.gci_new_symbol(&value)? };
        self.check_oop("new_symbol", oop)?;
        Ok(Oop(oop))
    }

    pub fn new_object(&mut self, class_oop: Oop) -> Result<Oop> {
        self.activate()?;
        let oop = unsafe { self.lib.gci_new_oop(class_oop.raw())? };
        self.check_oop("new_object", oop)?;
        Ok(Oop(oop))
    }

    pub fn smallint_oop(&self, value: i64) -> Oop {
        Oop::from_smallint(value)
    }

    pub fn bool_oop(&self, value: bool) -> Oop {
        Oop::from_bool(value)
    }

    pub fn nil_oop(&self) -> Oop {
        Oop::NIL
    }

    pub fn float_oop(&mut self, value: f64) -> Result<Oop> {
        self.activate()?;
        let oop = unsafe { self.lib.gci_flt_to_oop(value as c_double)? };
        if oop == OOP_ILLEGAL || oop == OOP_NIL {
            return Err(Error::IllegalOop {
                operation: "float_oop",
            });
        }
        Ok(Oop(oop))
    }

    pub fn try_oop_to_float(&mut self, oop: Oop) -> Result<Option<f64>> {
        self.activate()?;
        let mut value: c_double = 0.0;
        let ok = unsafe {
            self.lib
                .gci_oop_to_flt(oop.raw(), &mut value as *mut c_double)?
        };
        Ok((ok != 0).then_some(value as f64))
    }

    pub fn value_to_oop(&mut self, value: &Value) -> Result<Oop> {
        match value {
            Value::Nil => Ok(Oop::NIL),
            Value::Bool(value) => Ok(Oop::from_bool(*value)),
            Value::SmallInt(value) => Ok(Oop::from_smallint(*value)),
            Value::Char(value) => Ok(Oop::from_char(*value)),
            Value::String(value) => self.new_string(value),
            Value::Oop(oop) => Ok(*oop),
        }
    }

    pub fn identity_for_oop(&self, oop: Oop) -> usize {
        let mut identity_map = self.identity_map.borrow_mut();
        if let Some(identity) = identity_map.get(&oop.raw()).copied() {
            return identity;
        }
        let identity = self.next_identity.get();
        self.next_identity.set(identity.saturating_add(1));
        identity_map.insert(oop.raw(), identity);
        identity
    }

    pub fn cached_identity_for_oop(&self, oop: Oop) -> Option<usize> {
        self.identity_map.borrow().get(&oop.raw()).copied()
    }

    pub fn identity_map_len(&self) -> usize {
        self.identity_map.borrow().len()
    }

    pub fn clear_identity_map(&self) {
        self.identity_map.borrow_mut().clear();
        self.next_identity.set(1);
    }

    pub fn fetch_size(&mut self, oop: Oop) -> Result<i64> {
        self.activate()?;
        Ok(unsafe { self.lib.gci_fetch_size(oop.raw())? })
    }

    pub fn fetch_string(&mut self, oop: Oop) -> Result<String> {
        let size = self.fetch_size(oop)?;
        if size < 0 {
            return Err(Error::NegativeSize(size));
        }
        if size == 0 {
            return Ok(String::new());
        }

        let mut buffer = vec![0 as c_char; size as usize + 1];
        self.activate()?;
        let fetched = unsafe {
            self.lib
                .gci_fetch_bytes(oop.raw(), 1, buffer.as_mut_ptr(), size)?
        };
        if fetched < 0 {
            return Err(Error::NegativeSize(fetched));
        }
        let bytes: Vec<u8> = buffer
            .iter()
            .take(fetched as usize)
            .map(|byte| *byte as u8)
            .collect();
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    pub fn fetch_class(&mut self, oop: Oop) -> Result<Oop> {
        self.activate()?;
        let class_oop = unsafe { self.lib.gci_fetch_class(oop.raw())? };
        self.check_oop("fetch_class", class_oop)?;
        Ok(Oop(class_oop))
    }

    pub fn array_oops(&mut self, oop: Oop) -> Result<Vec<Oop>> {
        let size = self.fetch_size(oop)?;
        if size < 0 {
            return Err(Error::NegativeSize(size));
        }
        let mut values = Vec::with_capacity(size as usize);
        for index in 1..=size {
            let index = self.smallint_oop(index);
            values.push(self.perform_oop(oop, "at:", &[index])?);
        }
        Ok(values)
    }

    pub fn array_strings(&mut self, oop: Oop) -> Result<Vec<String>> {
        let values = self.array_oops(oop)?;
        values
            .into_iter()
            .map(|value| self.fetch_string(value))
            .collect()
    }

    pub fn global_get(&mut self, symbol_name: &str) -> Result<Oop> {
        let user_globals = self.resolve("UserGlobals")?;
        let key = CString::new(symbol_name)?;
        let mut value = OOP_ILLEGAL;
        let mut assoc = OOP_ILLEGAL;
        self.activate()?;
        unsafe {
            self.lib.gci_sym_dict_at(
                user_globals.raw(),
                &key,
                &mut value as *mut RawOop,
                &mut assoc as *mut RawOop,
            )?;
        }
        self.check_oop("global_get", value)?;
        Ok(Oop(value))
    }

    pub fn global_put(&mut self, symbol_name: &str, value: Oop) -> Result<()> {
        let user_globals = self.resolve("UserGlobals")?;
        let key = self.new_symbol(symbol_name)?;
        self.activate()?;
        unsafe {
            self.lib
                .gci_sym_dict_at_obj_put(user_globals.raw(), key.raw(), value.raw())?;
        }
        Ok(())
    }

    pub fn str_dict_get(&mut self, dict: Oop, key: &str) -> Result<Value> {
        let key = CString::new(key)?;
        let mut value = OOP_ILLEGAL;
        self.activate()?;
        unsafe {
            self.lib
                .gci_str_key_value_dict_at(dict.raw(), &key, &mut value as *mut RawOop)?;
        }
        self.check_oop("str_dict_get", value)?;
        self.marshal(Oop(value))
    }

    pub fn str_dict_put(&mut self, dict: Oop, key: &str, value: Oop) -> Result<()> {
        let key = CString::new(key)?;
        self.activate()?;
        unsafe {
            self.lib
                .gci_str_key_value_dict_at_put(dict.raw(), &key, value.raw())?;
        }
        Ok(())
    }

    pub fn retain_oop(&mut self, oop: Oop) -> Result<OopHandle<'_>> {
        self.add_to_export_set(oop)?;
        Ok(OopHandle {
            session: self,
            oop,
            released: false,
        })
    }

    pub fn add_to_export_set(&mut self, oop: Oop) -> Result<()> {
        self.activate()?;
        unsafe {
            if !self
                .lib
                .call_optional_oop_export(b"GciAddOopToExportSet", oop.raw())?
            {
                self.lib
                    .call_optional_oop_export(b"GciAddObjToExportSet", oop.raw())?;
            }
        }
        Ok(())
    }

    pub fn remove_from_export_set(&mut self, oop: Oop) -> Result<()> {
        if !self.logged_in.get() {
            return Ok(());
        }
        self.activate()?;
        unsafe {
            if !self
                .lib
                .call_optional_oop_export(b"GciRemoveOopFromExportSet", oop.raw())?
            {
                self.lib
                    .call_optional_oop_export(b"GciRemoveObjFromExportSet", oop.raw())?;
            }
        }
        Ok(())
    }

    pub fn needs_commit(&mut self) -> Result<bool> {
        self.activate()?;
        Ok(unsafe { self.lib.gci_needs_commit()? != 0 })
    }

    pub fn in_transaction(&mut self) -> Result<bool> {
        self.activate()?;
        Ok(unsafe { self.lib.gci_in_transaction()? != 0 })
    }

    pub fn commit(&mut self) -> Result<()> {
        self.activate()?;
        let mut err = GciErrSType::default();
        let ok = unsafe { self.lib.gci_commit(&mut err)? };
        if ok == 0 && err.number != 0 {
            return Err(gemstone_error(err));
        }
        Ok(())
    }

    pub fn abort(&mut self) -> Result<()> {
        self.activate()?;
        let mut err = GciErrSType::default();
        let ok = unsafe { self.lib.gci_abort(&mut err)? };
        if ok == 0 && err.number != 0 {
            return Err(gemstone_error(err));
        }
        Ok(())
    }

    pub fn transaction<T>(&mut self, body: impl FnOnce(&mut Session) -> Result<T>) -> Result<T> {
        match body(self) {
            Ok(value) => {
                self.commit()?;
                Ok(value)
            }
            Err(err) => {
                let _ = self.abort();
                Err(err)
            }
        }
    }

    pub fn commit_with_retry(&mut self, retries: usize) -> Result<()> {
        let mut attempts = 0;
        loop {
            match self.commit() {
                Ok(()) => return Ok(()),
                Err(err) if attempts < retries => {
                    attempts += 1;
                    let _ = self.abort();
                    if err_is_fatal(&err) {
                        return Err(err);
                    }
                }
                Err(err) => return Err(err),
            }
        }
    }

    pub fn bridge_root(&mut self) -> Result<BridgeRoot<'_>> {
        BridgeRoot::new(self)
    }

    pub fn bridge_root_named(&mut self, name: impl Into<String>) -> Result<BridgeRoot<'_>> {
        BridgeRoot::named(self, name)
    }

    pub fn logout(&mut self) -> Result<()> {
        if !self.logged_in.get() {
            return Ok(());
        }
        unsafe {
            self.lib.gci_set_session_id(self.session_id)?;
            self.lib.gci_logout()?;
        }
        self.logged_in.set(false);
        self.session_id = GCI_INVALID_SESSION as i32;
        Ok(())
    }

    fn open(&mut self, config: Config) -> Result<()> {
        unsafe {
            self.lib.gci_init()?;
        }

        let stone_nrs = CString::new(config.stone_nrs())?;
        let host_username = CString::new(config.host_username)?;
        let host_password = CString::new(config.host_password)?;
        let gem_service = CString::new(config.gem_service)?;
        let username = CString::new(config.username)?;
        let password = CString::new(config.password)?;

        let mut encrypted_host_password = vec![0 as c_char; GCI_ENCRYPT_BUF_SIZE];
        unsafe {
            self.lib.gci_encrypt(
                &host_password,
                encrypted_host_password.as_mut_ptr(),
                GCI_ENCRYPT_BUF_SIZE as c_uint,
            )?;
            self.lib.gci_set_net(
                &stone_nrs,
                &host_username,
                encrypted_host_password.as_ptr(),
                &gem_service,
            )?;
            let ok = self.lib.gci_login_ex(&username, &password, 0, 0)?;
            if ok == 0 {
                let mut err = GciErrSType::default();
                self.lib.gci_err(&mut err)?;
                return Err(gemstone_error(err));
            }
            self.session_id = self.lib.gci_get_session_id()?;
        }
        self.logged_in.set(true);
        Ok(())
    }

    fn activate(&self) -> Result<()> {
        if !self.logged_in.get() {
            return Err(Error::NotLoggedIn);
        }
        unsafe {
            self.lib.gci_set_session_id(self.session_id)?;
        }
        Ok(())
    }

    fn marshal(&self, oop: Oop) -> Result<Value> {
        match oop.raw() {
            OOP_NIL => Ok(Value::Nil),
            OOP_TRUE => Ok(Value::Bool(true)),
            OOP_FALSE => Ok(Value::Bool(false)),
            raw if is_smallint(raw) => Ok(Value::SmallInt(gemstone_gci::smallint_to_i64(raw))),
            raw if is_char(raw) => Ok(Value::Char(char_from_oop(raw)?)),
            _ => Ok(Value::Oop(oop)),
        }
    }

    fn check_oop(&self, operation: &'static str, oop: RawOop) -> Result<()> {
        if oop == OOP_ILLEGAL {
            Err(Error::IllegalOop { operation })
        } else {
            Ok(())
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.logout();
    }
}

pub use gemstone_gci::{OOP_FALSE, OOP_ILLEGAL, OOP_NIL, OOP_TRUE};

pub struct OopHandle<'a> {
    session: &'a mut Session,
    oop: Oop,
    released: bool,
}

impl<'a> OopHandle<'a> {
    pub fn oop(&self) -> Oop {
        self.oop
    }

    pub fn release(mut self) -> Result<()> {
        self.release_now()
    }

    fn release_now(&mut self) -> Result<()> {
        if self.released {
            return Ok(());
        }
        self.session.remove_from_export_set(self.oop)?;
        self.released = true;
        Ok(())
    }
}

impl Drop for OopHandle<'_> {
    fn drop(&mut self) {
        let _ = self.release_now();
    }
}

fn required_env(name: &'static str) -> Result<String> {
    env::var(name)
        .ok()
        .filter(|value| !value.is_empty())
        .ok_or(Error::MissingEnvironment(name))
}

fn required_config(name: &'static str, value: Option<String>) -> Result<String> {
    value
        .filter(|value| !value.is_empty())
        .ok_or(Error::MissingConfig(name))
}

fn gemstone_error(err: GciErrSType) -> Error {
    Error::GemStone {
        number: err.number,
        fatal: err.fatal != 0,
        message: err.full_message(),
    }
}

fn err_is_fatal(err: &Error) -> bool {
    matches!(err, Error::GemStone { fatal: true, .. })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard};

    static LIVE_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn config_formats_remote_stone_nrs() {
        let config = Config {
            stone: "seaside".to_string(),
            netldi: "netldi".to_string(),
            host: "db.example.test".to_string(),
            username: "DataCurator".to_string(),
            password: "swordfish".to_string(),
            ..Config::default()
        };

        assert_eq!(config.stone_nrs(), "!@db.example.test!netldi!seaside");
    }

    #[test]
    fn gci_library_resolution_reports_explicit_config_path() -> Result<()> {
        let config = Config {
            lib_path: Some(PathBuf::from("/tmp/libgcirpc-test.dylib")),
            ..Config::default()
        };

        let resolution = gci_library_resolution(&config)?;

        assert_eq!(resolution.source, "config.lib_path");
        assert_eq!(resolution.path, PathBuf::from("/tmp/libgcirpc-test.dylib"));
        assert_eq!(
            resolution.searched,
            vec![PathBuf::from("/tmp/libgcirpc-test.dylib")]
        );
        Ok(())
    }

    #[test]
    fn config_builder_applies_defaults_and_required_fields() -> Result<()> {
        let config = Config::builder()
            .username("DataCurator")
            .password("swordfish")
            .stone("seaside")
            .build()?;

        assert_eq!(config.stone, "seaside");
        assert_eq!(config.host, "localhost");
        assert_eq!(config.netldi, "netldi");
        assert_eq!(config.gem_service, "gemnetobject");
        Ok(())
    }

    #[test]
    fn config_builder_requires_credentials() {
        let err = Config::builder().password("swordfish").build().unwrap_err();
        assert!(matches!(err, Error::MissingConfig("username")));
    }

    #[test]
    fn smallint_values_marshal_without_a_live_session() {
        let value = Value::SmallInt(gemstone_gci::smallint_to_i64(
            gemstone_gci::i64_to_smallint(7),
        ));
        assert_eq!(value, Value::SmallInt(7));
    }

    #[test]
    fn value_oop_accessors_are_explicit() {
        let oop = Oop::from_smallint(7);
        assert_eq!(Value::Oop(oop).as_oop(), Some(oop));
        assert_eq!(Value::SmallInt(7).as_oop(), None);
        assert_eq!(Oop::from_bool(true).as_bool(), Some(true));
        assert_eq!(Oop::from_char('A').as_char().unwrap(), Some('A'));
    }

    #[test]
    fn bridge_values_are_plain_rust_data_until_mapped() {
        let value = BridgeValue::dictionary([
            ("name".to_string(), BridgeValue::from("Tariq")),
            ("amount".to_string(), BridgeValue::from(100_i64)),
            ("currency".to_string(), BridgeValue::from("GBP")),
        ]);
        let BridgeValue::Dictionary(entries) = value else {
            panic!("expected dictionary bridge value");
        };
        assert_eq!(entries["name"], BridgeValue::String("Tariq".to_string()));
        assert_eq!(entries["amount"], BridgeValue::SmallInt(100));
    }

    #[derive(Debug, Eq, PartialEq)]
    struct TestBookingDraft {
        name: String,
        amount: i64,
        currency: String,
    }

    impl BridgeMapped for TestBookingDraft {
        fn to_bridge_value(&self) -> BridgeValue {
            BridgeValue::dictionary([
                ("name".to_string(), BridgeValue::from(self.name.clone())),
                ("amount".to_string(), BridgeValue::from(self.amount)),
                (
                    "currency".to_string(),
                    BridgeValue::from(self.currency.clone()),
                ),
            ])
        }

        fn from_bridge_dictionary(dictionary: &mut BridgeDictionary<'_>) -> Result<Self> {
            Ok(Self {
                name: dictionary.at_string("name")?,
                amount: dictionary.at_smallint("amount")?,
                currency: dictionary.at_string("currency")?,
            })
        }
    }

    #[derive(Clone, Debug, Eq, PartialEq, BridgeMapped)]
    struct DeriveCustomerDraft {
        #[bridge(key_type = "Symbol")]
        name: String,
    }

    #[derive(Clone, Debug, Eq, PartialEq, BridgeMapped)]
    struct DeriveBookingDraft {
        #[bridge(key = "amount", key_type = "Symbol")]
        amount: i64,
        customer: DeriveCustomerDraft,
        tags: Vec<String>,
        note: Option<String>,
    }

    #[test]
    fn bridge_mapped_derive_supports_key_policy_and_nested_values() {
        let value = DeriveBookingDraft {
            amount: 100,
            customer: DeriveCustomerDraft {
                name: "Tariq".to_string(),
            },
            tags: vec!["priority".to_string(), "demo".to_string()],
            note: None,
        }
        .to_bridge_value();

        let BridgeValue::KeyedDictionary(entries) = value else {
            panic!("expected keyed dictionary");
        };
        assert_eq!(entries[0].0.name, "amount");
        assert_eq!(entries[0].0.key_type, BridgeKeyType::Symbol);
        assert!(matches!(entries[1].1, BridgeValue::KeyedDictionary(_)));
        assert!(matches!(entries[2].1, BridgeValue::Array(_)));
        assert!(matches!(entries[3].1, BridgeValue::Nil));
    }

    #[test]
    fn live_eval_smoke_returns_seven_when_enabled() -> Result<()> {
        let Some(_guard) = live_test_guard() else {
            return Ok(());
        };

        let Some(mut session) = live_session()? else {
            return Ok(());
        };
        assert_eq!(session.eval("3 + 4")?, Value::SmallInt(7));
        session.logout()?;
        Ok(())
    }

    #[test]
    fn live_login_logout_when_enabled() -> Result<()> {
        let Some(_guard) = live_test_guard() else {
            return Ok(());
        };
        let Some(mut session) = live_session()? else {
            return Ok(());
        };

        assert!(session.is_logged_in());
        session.logout()?;
        assert!(!session.is_logged_in());
        Ok(())
    }

    #[test]
    fn live_perform_when_enabled() -> Result<()> {
        let Some(_guard) = live_test_guard() else {
            return Ok(());
        };
        let Some(mut session) = live_session()? else {
            return Ok(());
        };

        let seven = session.smallint_oop(7);
        let printed = session.perform_oop(seven, "printString", &[])?;
        assert_eq!(session.fetch_string(printed)?, "7");
        Ok(())
    }

    #[test]
    fn live_global_string_round_trip_when_enabled() -> Result<()> {
        let Some(_guard) = live_test_guard() else {
            return Ok(());
        };
        let Some(mut session) = live_session()? else {
            return Ok(());
        };

        let key = live_key("GemStoneRsRoundTrip");
        let text = session.new_string("hello from gemstone-rs")?;
        session.global_put(&key, text)?;
        let stored = session.global_get(&key)?;
        assert_eq!(session.fetch_string(stored)?, "hello from gemstone-rs");
        session.global_put(&key, Oop::NIL)?;
        session.commit()?;
        Ok(())
    }

    #[test]
    fn live_error_handling_when_enabled() -> Result<()> {
        let Some(_guard) = live_test_guard() else {
            return Ok(());
        };
        let Some(mut session) = live_session()? else {
            return Ok(());
        };

        session.logout()?;
        let err = session.eval("3 + 4").unwrap_err();
        assert!(matches!(err, Error::NotLoggedIn));
        Ok(())
    }

    #[test]
    fn live_transaction_commit_and_abort_when_enabled() -> Result<()> {
        let Some(_guard) = live_test_guard() else {
            return Ok(());
        };
        let Some(mut session) = live_session()? else {
            return Ok(());
        };

        let committed_key = live_key("GemStoneRsCommitted");
        session.transaction(|session| {
            let value = session.new_string("committed")?;
            session.global_put(&committed_key, value)
        })?;
        let committed = session.global_get(&committed_key)?;
        assert_eq!(session.fetch_string(committed)?, "committed");

        let aborted_key = live_key("GemStoneRsAborted");
        let aborted_value = session.new_string("aborted")?;
        session.global_put(&aborted_key, aborted_value)?;
        session.abort()?;
        assert!(session.global_get(&aborted_key).is_err());

        session.global_put(&committed_key, Oop::NIL)?;
        session.commit()?;
        Ok(())
    }

    #[test]
    fn live_bridge_root_mapping_when_enabled() -> Result<()> {
        let Some(_guard) = live_test_guard() else {
            return Ok(());
        };
        let Some(mut session) = live_session()? else {
            return Ok(());
        };

        let key = live_key("GemStoneRsBridgePayload");
        let payload = TestBookingDraft {
            name: "Tariq".to_string(),
            amount: 100,
            currency: "GBP".to_string(),
        };
        let mut bridge_root = session.bridge_root()?;
        let payload_oop = bridge_root.put_mapped(&key, &payload)?;
        let stored = bridge_root.get_oop(&key)?;
        assert_eq!(payload_oop, stored);
        let keys = bridge_root.keys()?;
        assert!(keys.iter().any(|entry| entry.print_string.contains(&key)));
        let loaded: TestBookingDraft = bridge_root.get_mapped(&key)?;
        assert_eq!(loaded, payload);
        bridge_root.remove(&key)?;
        bridge_root.commit()?;
        Ok(())
    }

    fn live_session() -> Result<Option<Session>> {
        if env::var("GS_RUN_LIVE_RUST").as_deref() != Ok("1") {
            return Ok(None);
        }
        let config = match Config::from_env() {
            Ok(config) => config,
            Err(Error::MissingEnvironment(name)) => {
                eprintln!("skipping live GemStone test: {name} is not set");
                return Ok(None);
            }
            Err(err) => return Err(err),
        };
        Session::login(config).map(Some)
    }

    fn live_test_guard() -> Option<MutexGuard<'static, ()>> {
        if env::var("GS_RUN_LIVE_RUST").as_deref() == Ok("1") {
            Some(
                LIVE_TEST_LOCK
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()),
            )
        } else {
            None
        }
    }

    fn live_key(prefix: &str) -> String {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|value| value.as_nanos())
            .unwrap_or_default();
        format!("{prefix}{nanos}")
    }
}
