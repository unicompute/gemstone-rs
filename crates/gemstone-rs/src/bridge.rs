use crate::{Error, Oop, Result, Session, Value};
use std::collections::BTreeMap;

pub const DEFAULT_BRIDGE_ROOT: &str = "GemStoneRsBridgeRoot";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeKeyType {
    String,
    Symbol,
}

impl BridgeKeyType {
    pub fn config_name(self) -> &'static str {
        match self {
            Self::String => "String",
            Self::Symbol => "Symbol",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeKey {
    pub name: String,
    pub key_type: BridgeKeyType,
}

impl BridgeKey {
    pub fn string(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            key_type: BridgeKeyType::String,
        }
    }

    pub fn symbol(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            key_type: BridgeKeyType::Symbol,
        }
    }

    pub fn new(name: impl Into<String>, key_type: BridgeKeyType) -> Self {
        Self {
            name: name.into(),
            key_type,
        }
    }

    fn to_oop(&self, session: &mut Session) -> Result<Oop> {
        match self.key_type {
            BridgeKeyType::String => session.new_string(&self.name),
            BridgeKeyType::Symbol => session.new_symbol(&self.name),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BridgeValue {
    Nil,
    Bool(bool),
    SmallInt(i64),
    String(String),
    Symbol(String),
    Oop(Oop),
    Dictionary(BTreeMap<String, BridgeValue>),
    KeyedDictionary(Vec<(BridgeKey, BridgeValue)>),
    Array(Vec<BridgeValue>),
}

impl BridgeValue {
    pub fn dictionary(entries: impl IntoIterator<Item = (String, BridgeValue)>) -> Self {
        Self::Dictionary(entries.into_iter().collect())
    }

    pub fn keyed_dictionary(entries: impl IntoIterator<Item = (BridgeKey, BridgeValue)>) -> Self {
        Self::KeyedDictionary(entries.into_iter().collect())
    }

    pub fn array(values: impl IntoIterator<Item = BridgeValue>) -> Self {
        Self::Array(values.into_iter().collect())
    }

    pub fn to_oop(&self, session: &mut Session) -> Result<Oop> {
        match self {
            Self::Nil => Ok(session.nil_oop()),
            Self::Bool(value) => Ok(session.bool_oop(*value)),
            Self::SmallInt(value) => Ok(session.smallint_oop(*value)),
            Self::String(value) => session.new_string(value),
            Self::Symbol(value) => session.new_symbol(value),
            Self::Oop(oop) => Ok(*oop),
            Self::Dictionary(entries) => {
                let dictionary = session.execute("Dictionary new")?;
                session.identity_for_oop(dictionary);
                for (key, value) in entries {
                    let key = session.new_string(key)?;
                    let value = value.to_oop(session)?;
                    session.perform_oop(dictionary, "at:put:", &[key, value])?;
                }
                Ok(dictionary)
            }
            Self::KeyedDictionary(entries) => {
                let dictionary = session.execute("Dictionary new")?;
                session.identity_for_oop(dictionary);
                for (key, value) in entries {
                    let key = key.to_oop(session)?;
                    let value = value.to_oop(session)?;
                    session.perform_oop(dictionary, "at:put:", &[key, value])?;
                }
                Ok(dictionary)
            }
            Self::Array(values) => {
                let array = session.execute(&format!("Array new: {}", values.len()))?;
                session.identity_for_oop(array);
                for (index, value) in values.iter().enumerate() {
                    let index = session.smallint_oop((index + 1) as i64);
                    let value = value.to_oop(session)?;
                    session.perform_oop(array, "at:put:", &[index, value])?;
                }
                Ok(array)
            }
        }
    }
}

impl From<bool> for BridgeValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i64> for BridgeValue {
    fn from(value: i64) -> Self {
        Self::SmallInt(value)
    }
}

impl From<i32> for BridgeValue {
    fn from(value: i32) -> Self {
        Self::SmallInt(i64::from(value))
    }
}

impl From<&str> for BridgeValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<String> for BridgeValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<Oop> for BridgeValue {
    fn from(value: Oop) -> Self {
        Self::Oop(value)
    }
}

impl From<Value> for BridgeValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Nil => Self::Nil,
            Value::Bool(value) => Self::Bool(value),
            Value::SmallInt(value) => Self::SmallInt(value),
            Value::Char(value) => Self::String(value.to_string()),
            Value::String(value) => Self::String(value),
            Value::Oop(oop) => Self::Oop(oop),
        }
    }
}

pub trait BridgeFieldWrite {
    fn to_bridge_field_value(&self) -> BridgeValue;
}

impl BridgeFieldWrite for bool {
    fn to_bridge_field_value(&self) -> BridgeValue {
        BridgeValue::Bool(*self)
    }
}

impl BridgeFieldWrite for i64 {
    fn to_bridge_field_value(&self) -> BridgeValue {
        BridgeValue::SmallInt(*self)
    }
}

impl BridgeFieldWrite for i32 {
    fn to_bridge_field_value(&self) -> BridgeValue {
        BridgeValue::SmallInt(i64::from(*self))
    }
}

impl BridgeFieldWrite for String {
    fn to_bridge_field_value(&self) -> BridgeValue {
        BridgeValue::String(self.clone())
    }
}

impl BridgeFieldWrite for &str {
    fn to_bridge_field_value(&self) -> BridgeValue {
        BridgeValue::String((*self).to_string())
    }
}

impl BridgeFieldWrite for Oop {
    fn to_bridge_field_value(&self) -> BridgeValue {
        BridgeValue::Oop(*self)
    }
}

impl BridgeFieldWrite for BridgeValue {
    fn to_bridge_field_value(&self) -> BridgeValue {
        self.clone()
    }
}

impl<T: BridgeMapped> BridgeFieldWrite for T {
    fn to_bridge_field_value(&self) -> BridgeValue {
        self.to_bridge_value()
    }
}

impl<T: BridgeFieldWrite> BridgeFieldWrite for Vec<T> {
    fn to_bridge_field_value(&self) -> BridgeValue {
        BridgeValue::array(self.iter().map(BridgeFieldWrite::to_bridge_field_value))
    }
}

pub trait BridgeFieldRead: Sized {
    fn read_bridge_field(
        dictionary: &mut BridgeDictionary<'_>,
        key: &str,
        key_type: BridgeKeyType,
    ) -> Result<Self> {
        let oop = dictionary.at_oop_with_key_type(key, key_type)?;
        Self::read_bridge_oop(
            dictionary.session,
            oop,
            &BridgeFieldContext::new(key, key_type, Self::expected_type()),
        )
    }

    fn read_bridge_oop(
        session: &mut Session,
        oop: Oop,
        context: &BridgeFieldContext,
    ) -> Result<Self>;

    fn expected_type() -> &'static str;
}

impl BridgeFieldRead for String {
    fn read_bridge_oop(
        session: &mut Session,
        oop: Oop,
        _context: &BridgeFieldContext,
    ) -> Result<Self> {
        session.fetch_string(oop)
    }

    fn expected_type() -> &'static str {
        "String"
    }
}

impl BridgeFieldRead for i64 {
    fn read_bridge_oop(
        _session: &mut Session,
        oop: Oop,
        context: &BridgeFieldContext,
    ) -> Result<Self> {
        oop.as_smallint()
            .ok_or_else(|| context.unexpected(format!("OOP {}", oop.raw())))
    }

    fn expected_type() -> &'static str {
        "SmallInt"
    }
}

impl BridgeFieldRead for bool {
    fn read_bridge_oop(
        _session: &mut Session,
        oop: Oop,
        context: &BridgeFieldContext,
    ) -> Result<Self> {
        oop.as_bool()
            .ok_or_else(|| context.unexpected(format!("OOP {}", oop.raw())))
    }

    fn expected_type() -> &'static str {
        "Bool"
    }
}

impl BridgeFieldRead for Oop {
    fn read_bridge_oop(
        session: &mut Session,
        oop: Oop,
        _context: &BridgeFieldContext,
    ) -> Result<Self> {
        session.identity_for_oop(oop);
        Ok(oop)
    }

    fn expected_type() -> &'static str {
        "Oop"
    }
}

impl<T: BridgeMapped> BridgeFieldRead for T {
    fn read_bridge_oop(
        session: &mut Session,
        oop: Oop,
        _context: &BridgeFieldContext,
    ) -> Result<Self> {
        let mut dictionary = BridgeDictionary::from_oop(session, oop);
        T::from_bridge_dictionary(&mut dictionary)
    }

    fn expected_type() -> &'static str {
        "Dictionary"
    }
}

impl<T: BridgeFieldRead> BridgeFieldRead for Vec<T> {
    fn read_bridge_oop(
        session: &mut Session,
        oop: Oop,
        context: &BridgeFieldContext,
    ) -> Result<Self> {
        let size = session.fetch_size(oop)?;
        if size < 0 {
            return Err(Error::NegativeSize(size));
        }
        let mut values = Vec::with_capacity(size as usize);
        for index in 1..=size {
            let index_oop = session.smallint_oop(index);
            let value_oop = session.perform_oop(oop, "at:", &[index_oop])?;
            values.push(T::read_bridge_oop(session, value_oop, context)?);
        }
        Ok(values)
    }

    fn expected_type() -> &'static str {
        "Array"
    }
}

pub struct BridgeRoot<'a> {
    session: &'a mut Session,
    name: String,
    oop: Oop,
}

impl<'a> BridgeRoot<'a> {
    pub fn new(session: &'a mut Session) -> Result<Self> {
        Self::named(session, DEFAULT_BRIDGE_ROOT)
    }

    pub fn named(session: &'a mut Session, name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        let oop = match session.global_get(&name) {
            Ok(oop) if oop != session.nil_oop() => oop,
            Ok(_) | Err(_) => {
                let root = session.execute("Dictionary new")?;
                session.global_put(&name, root)?;
                root
            }
        };
        session.identity_for_oop(oop);
        Ok(Self { session, name, oop })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn oop(&self) -> Oop {
        self.oop
    }

    pub fn identity_id(&self) -> usize {
        self.session.identity_for_oop(self.oop)
    }

    pub fn put(&mut self, key: &str, value: impl Into<BridgeValue>) -> Result<Oop> {
        self.put_with_key_type(key, BridgeKeyType::String, value)
    }

    pub fn put_with_key_type(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
        value: impl Into<BridgeValue>,
    ) -> Result<Oop> {
        let key_oop = BridgeKey::new(key, key_type).to_oop(self.session)?;
        let value = value.into().to_oop(self.session)?;
        self.session
            .perform_oop(self.oop, "at:put:", &[key_oop, value])?;
        self.session.identity_for_oop(value);
        Ok(value)
    }

    pub fn put_mapped(&mut self, key: &str, value: &impl BridgeMapped) -> Result<Oop> {
        self.put(key, value.to_bridge_value())
    }

    pub fn get_oop(&mut self, key: &str) -> Result<Oop> {
        self.get_oop_with_key_type(key, BridgeKeyType::String)
    }

    pub fn get_oop_with_key_type(&mut self, key: &str, key_type: BridgeKeyType) -> Result<Oop> {
        let key_oop = BridgeKey::new(key, key_type).to_oop(self.session)?;
        let oop = self.session.perform_oop(self.oop, "at:", &[key_oop])?;
        self.session.identity_for_oop(oop);
        Ok(oop)
    }

    pub fn get_value(&mut self, key: &str) -> Result<Value> {
        self.get_value_with_key_type(key, BridgeKeyType::String)
    }

    pub fn get_value_with_key_type(&mut self, key: &str, key_type: BridgeKeyType) -> Result<Value> {
        let key_oop = BridgeKey::new(key, key_type).to_oop(self.session)?;
        self.session.perform(self.oop, "at:", &[key_oop])
    }

    pub fn get_string(&mut self, key: &str) -> Result<String> {
        let oop = self.get_oop(key)?;
        self.session.fetch_string(oop)
    }

    pub fn get_smallint(&mut self, key: &str) -> Result<i64> {
        match self.get_value(key)? {
            Value::SmallInt(value) => Ok(value),
            other => Err(unexpected_field(key, "SmallInt", other)),
        }
    }

    pub fn get_bool(&mut self, key: &str) -> Result<bool> {
        match self.get_value(key)? {
            Value::Bool(value) => Ok(value),
            other => Err(unexpected_field(key, "Bool", other)),
        }
    }

    pub fn get_dictionary(&mut self, key: &str) -> Result<BridgeDictionary<'_>> {
        let oop = self.get_oop(key)?;
        Ok(BridgeDictionary::from_oop(self.session, oop))
    }

    pub fn get_mapped<T: BridgeMapped>(&mut self, key: &str) -> Result<T> {
        let mut dictionary = self.get_dictionary(key)?;
        T::from_bridge_dictionary(&mut dictionary)
    }

    pub fn remove(&mut self, key: &str) -> Result<Oop> {
        self.remove_with_key_type(key, BridgeKeyType::String)
    }

    pub fn remove_with_key_type(&mut self, key: &str, key_type: BridgeKeyType) -> Result<Oop> {
        let key_oop = BridgeKey::new(key, key_type).to_oop(self.session)?;
        self.session.perform_oop(
            self.oop,
            "removeKey:ifAbsent:",
            &[key_oop, self.session.nil_oop()],
        )
    }

    pub fn commit(&mut self) -> Result<()> {
        self.session.commit()
    }

    pub fn commit_with_retry(&mut self, retries: usize) -> Result<()> {
        self.session.commit_with_retry(retries)
    }

    pub fn transaction<T>(
        &mut self,
        body: impl FnOnce(&mut BridgeRoot<'_>) -> Result<T>,
    ) -> Result<T> {
        match body(self) {
            Ok(value) => {
                self.commit()?;
                Ok(value)
            }
            Err(err) => {
                let _ = self.session.abort();
                Err(err)
            }
        }
    }
}

pub struct BridgeDictionary<'a> {
    pub(crate) session: &'a mut Session,
    oop: Oop,
}

impl<'a> BridgeDictionary<'a> {
    pub fn from_oop(session: &'a mut Session, oop: Oop) -> Self {
        session.identity_for_oop(oop);
        Self { session, oop }
    }

    pub fn oop(&self) -> Oop {
        self.oop
    }

    pub fn identity_id(&self) -> usize {
        self.session.identity_for_oop(self.oop)
    }

    pub fn put(&mut self, key: &str, value: impl Into<BridgeValue>) -> Result<Oop> {
        self.put_with_key_type(key, BridgeKeyType::String, value)
    }

    pub fn put_with_key_type(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
        value: impl Into<BridgeValue>,
    ) -> Result<Oop> {
        let key_oop = BridgeKey::new(key, key_type).to_oop(self.session)?;
        let value = value.into().to_oop(self.session)?;
        self.session
            .perform_oop(self.oop, "at:put:", &[key_oop, value])?;
        self.session.identity_for_oop(value);
        Ok(value)
    }

    pub fn at_oop(&mut self, key: &str) -> Result<Oop> {
        self.at_oop_with_key_type(key, BridgeKeyType::String)
    }

    pub fn at_oop_with_key_type(&mut self, key: &str, key_type: BridgeKeyType) -> Result<Oop> {
        let key_oop = BridgeKey::new(key, key_type).to_oop(self.session)?;
        let oop = self.session.perform_oop(self.oop, "at:", &[key_oop])?;
        self.session.identity_for_oop(oop);
        Ok(oop)
    }

    pub fn at_value(&mut self, key: &str) -> Result<Value> {
        self.at_value_with_key_type(key, BridgeKeyType::String)
    }

    pub fn at_value_with_key_type(&mut self, key: &str, key_type: BridgeKeyType) -> Result<Value> {
        let key_oop = BridgeKey::new(key, key_type).to_oop(self.session)?;
        self.session.perform(self.oop, "at:", &[key_oop])
    }

    pub fn at_string(&mut self, key: &str) -> Result<String> {
        self.at_string_with_key_type(key, BridgeKeyType::String)
    }

    pub fn at_string_with_key_type(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
    ) -> Result<String> {
        let oop = self.at_oop_with_key_type(key, key_type)?;
        self.session.fetch_string(oop)
    }

    pub fn at_smallint(&mut self, key: &str) -> Result<i64> {
        self.at_smallint_with_key_type(key, BridgeKeyType::String)
    }

    pub fn at_smallint_with_key_type(&mut self, key: &str, key_type: BridgeKeyType) -> Result<i64> {
        match self.at_value_with_key_type(key, key_type)? {
            Value::SmallInt(value) => Ok(value),
            other => Err(unexpected_field(key, "SmallInt", other)),
        }
    }

    pub fn at_bool(&mut self, key: &str) -> Result<bool> {
        self.at_bool_with_key_type(key, BridgeKeyType::String)
    }

    pub fn at_bool_with_key_type(&mut self, key: &str, key_type: BridgeKeyType) -> Result<bool> {
        match self.at_value_with_key_type(key, key_type)? {
            Value::Bool(value) => Ok(value),
            other => Err(unexpected_field(key, "Bool", other)),
        }
    }

    pub fn at_dictionary(&mut self, key: &str) -> Result<BridgeDictionary<'_>> {
        self.at_dictionary_with_key_type(key, BridgeKeyType::String)
    }

    pub fn at_dictionary_with_key_type(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
    ) -> Result<BridgeDictionary<'_>> {
        let oop = self.at_oop_with_key_type(key, key_type)?;
        Ok(BridgeDictionary::from_oop(self.session, oop))
    }

    pub fn at_mapped<T: BridgeMapped>(&mut self, key: &str) -> Result<T> {
        self.at_mapped_with_key_type(key, BridgeKeyType::String)
    }

    pub fn at_mapped_with_key_type<T: BridgeMapped>(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
    ) -> Result<T> {
        let mut dictionary = self.at_dictionary_with_key_type(key, key_type)?;
        T::from_bridge_dictionary(&mut dictionary)
    }

    pub fn at_vec<T: BridgeFieldRead>(&mut self, key: &str) -> Result<Vec<T>> {
        self.at_vec_with_key_type(key, BridgeKeyType::String)
    }

    pub fn at_vec_with_key_type<T: BridgeFieldRead>(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
    ) -> Result<Vec<T>> {
        BridgeFieldRead::read_bridge_field(self, key, key_type)
    }
}

pub trait BridgeMapped: Sized {
    fn to_bridge_value(&self) -> BridgeValue;

    fn from_bridge_dictionary(dictionary: &mut BridgeDictionary<'_>) -> Result<Self>;
}

#[doc(hidden)]
pub struct BridgeFieldContext {
    key: String,
    key_type: BridgeKeyType,
    expected: &'static str,
}

impl BridgeFieldContext {
    fn new(key: &str, key_type: BridgeKeyType, expected: &'static str) -> Self {
        Self {
            key: key.to_string(),
            key_type,
            expected,
        }
    }

    fn unexpected(&self, actual: String) -> Error {
        Error::Mapping {
            field: format!("{} ({})", self.key, self.key_type.config_name()),
            expected: self.expected,
            actual,
        }
    }
}

fn unexpected_field(key: &str, expected: &'static str, actual: Value) -> Error {
    Error::Mapping {
        field: key.to_string(),
        expected,
        actual: format!("{actual:?}"),
    }
}
