use crate::{Error, Oop, Result, Session, Value};
use std::collections::{BTreeMap, BTreeSet};

pub const DEFAULT_BRIDGE_ROOT: &str = "GemStoneRsBridgeRoot";
pub const DEFAULT_BRIDGE_VALUE_DEPTH: usize = 8;

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BridgeKeySummary {
    pub oop: Oop,
    pub class_oop: Oop,
    pub print_string: String,
    pub identity_id: usize,
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

    pub fn from_oop(session: &mut Session, oop: Oop) -> Result<Self> {
        Self::from_oop_with_depth(session, oop, DEFAULT_BRIDGE_VALUE_DEPTH)
    }

    pub fn from_oop_with_depth(session: &mut Session, oop: Oop, max_depth: usize) -> Result<Self> {
        let mut seen = BTreeSet::new();
        Self::from_oop_inner(session, oop, max_depth, &mut seen)
    }

    fn from_oop_inner(
        session: &mut Session,
        oop: Oop,
        remaining_depth: usize,
        seen: &mut BTreeSet<u64>,
    ) -> Result<Self> {
        if oop.is_nil() {
            return Ok(Self::Nil);
        }
        if let Some(value) = oop.as_bool() {
            return Ok(Self::Bool(value));
        }
        if let Some(value) = oop.as_smallint() {
            return Ok(Self::SmallInt(value));
        }
        if let Some(value) = oop.as_char()? {
            return Ok(Self::String(value.to_string()));
        }

        session.identity_for_oop(oop);
        if remaining_depth == 0 || !seen.insert(oop.raw()) {
            return Ok(Self::Oop(oop));
        }

        let value = if session.is_kind_of(oop, "Symbol")? {
            Self::Symbol(session.fetch_string(oop)?)
        } else if session.is_kind_of(oop, "String")? {
            Self::String(session.fetch_string(oop)?)
        } else if session.is_kind_of(oop, "Array")? {
            let size = session.fetch_size(oop)?;
            if size < 0 {
                return Err(Error::NegativeSize(size));
            }
            let mut values = Vec::with_capacity(size as usize);
            for index in 1..=size {
                let index_oop = session.smallint_oop(index);
                let value_oop = session.perform_oop(oop, "at:", &[index_oop])?;
                values.push(Self::from_oop_inner(
                    session,
                    value_oop,
                    remaining_depth.saturating_sub(1),
                    seen,
                )?);
            }
            Self::Array(values)
        } else if session.is_kind_of(oop, "Dictionary")? {
            bridge_value_dictionary_from_oop(session, oop, remaining_depth, seen)?
        } else {
            Self::Oop(oop)
        };

        seen.remove(&oop.raw());
        Ok(value)
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

impl<T: BridgeFieldWrite> From<Vec<T>> for BridgeValue {
    fn from(value: Vec<T>) -> Self {
        value.to_bridge_field_value()
    }
}

impl<T: BridgeFieldWrite> From<BTreeMap<String, T>> for BridgeValue {
    fn from(value: BTreeMap<String, T>) -> Self {
        value.to_bridge_field_value()
    }
}

impl<T: BridgeFieldWrite> From<Option<T>> for BridgeValue {
    fn from(value: Option<T>) -> Self {
        value.to_bridge_field_value()
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

impl BridgeFieldWrite for str {
    fn to_bridge_field_value(&self) -> BridgeValue {
        BridgeValue::String(self.to_string())
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

impl<T: BridgeFieldWrite> BridgeFieldWrite for BTreeMap<String, T> {
    fn to_bridge_field_value(&self) -> BridgeValue {
        BridgeValue::dictionary(
            self.iter()
                .map(|(key, value)| (key.clone(), value.to_bridge_field_value())),
        )
    }
}

impl<T: BridgeFieldWrite> BridgeFieldWrite for Option<T> {
    fn to_bridge_field_value(&self) -> BridgeValue {
        self.as_ref()
            .map(BridgeFieldWrite::to_bridge_field_value)
            .unwrap_or(BridgeValue::Nil)
    }
}

pub trait BridgeFieldRead: Sized {
    fn read_bridge_field(
        dictionary: &mut BridgeDictionary<'_>,
        key: &str,
        key_type: BridgeKeyType,
    ) -> Result<Self> {
        let context = BridgeFieldContext::new(key, key_type, Self::expected_type());
        let oop = dictionary
            .at_oop_with_key_type(key, key_type)
            .map_err(|err| context.lookup_error(err))?;
        Self::read_bridge_oop(dictionary.session, oop, &context)
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
        context: &BridgeFieldContext,
    ) -> Result<Self> {
        session
            .fetch_string(oop)
            .map_err(|err| context.unexpected(format!("OOP {} ({err})", oop.raw())))
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
        context: &BridgeFieldContext,
    ) -> Result<Self> {
        let mut dictionary = BridgeDictionary::from_oop(session, oop);
        T::from_bridge_dictionary(&mut dictionary).map_err(|err| context.nested_error(err))
    }

    fn expected_type() -> &'static str {
        "Dictionary"
    }
}

impl BridgeFieldRead for BridgeValue {
    fn read_bridge_oop(
        session: &mut Session,
        oop: Oop,
        context: &BridgeFieldContext,
    ) -> Result<Self> {
        BridgeValue::from_oop(session, oop).map_err(|err| context.read_error(err))
    }

    fn expected_type() -> &'static str {
        "Any"
    }
}

impl<T: BridgeFieldRead> BridgeFieldRead for Vec<T> {
    fn read_bridge_oop(
        session: &mut Session,
        oop: Oop,
        context: &BridgeFieldContext,
    ) -> Result<Self> {
        let size = session
            .fetch_size(oop)
            .map_err(|err| context.unexpected(format!("OOP {} ({err})", oop.raw())))?;
        if size < 0 {
            return Err(context.unexpected(format!("negative array size {size}")));
        }
        let mut values = Vec::with_capacity(size as usize);
        for index in 1..=size {
            let index_oop = session.smallint_oop(index);
            let index_context = context.index(index, T::expected_type());
            let value_oop = session
                .perform_oop(oop, "at:", &[index_oop])
                .map_err(|err| index_context.lookup_error(err))?;
            values.push(T::read_bridge_oop(session, value_oop, &index_context)?);
        }
        Ok(values)
    }

    fn expected_type() -> &'static str {
        "Array"
    }
}

impl<T: BridgeFieldRead> BridgeFieldRead for BTreeMap<String, T> {
    fn read_bridge_oop(
        session: &mut Session,
        oop: Oop,
        context: &BridgeFieldContext,
    ) -> Result<Self> {
        dictionary_string_entries(session, oop, context)
    }

    fn expected_type() -> &'static str {
        "Dictionary"
    }
}

impl<T: BridgeFieldRead> BridgeFieldRead for Option<T> {
    fn read_bridge_field(
        dictionary: &mut BridgeDictionary<'_>,
        key: &str,
        key_type: BridgeKeyType,
    ) -> Result<Self> {
        let context = BridgeFieldContext::new(key, key_type, T::expected_type());
        if !dictionary
            .contains_key_with_key_type(key, key_type)
            .map_err(|err| context.lookup_error(err))?
        {
            return Ok(None);
        }
        let oop = dictionary
            .at_oop_with_key_type(key, key_type)
            .map_err(|err| context.lookup_error(err))?;
        Self::read_bridge_oop(dictionary.session, oop, &context)
    }

    fn read_bridge_oop(
        session: &mut Session,
        oop: Oop,
        context: &BridgeFieldContext,
    ) -> Result<Self> {
        if oop == session.nil_oop() {
            return Ok(None);
        }
        T::read_bridge_oop(session, oop, context).map(Some)
    }

    fn expected_type() -> &'static str {
        "Optional"
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

    pub fn keys(&mut self) -> Result<Vec<BridgeKeySummary>> {
        dictionary_keys(self.session, self.oop)
    }

    pub fn contains_key(&mut self, key: &str) -> Result<bool> {
        self.contains_key_with_key_type(key, BridgeKeyType::String)
    }

    pub fn contains_key_with_key_type(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
    ) -> Result<bool> {
        dictionary_contains_key(self.session, self.oop, key, key_type)
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

    pub fn put_mapped_with_key_type(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
        value: &impl BridgeMapped,
    ) -> Result<Oop> {
        self.put_with_key_type(key, key_type, value.to_bridge_value())
    }

    pub fn put_field<T: BridgeFieldWrite + ?Sized>(&mut self, key: &str, value: &T) -> Result<Oop> {
        self.put_field_with_key_type(key, BridgeKeyType::String, value)
    }

    pub fn put_field_with_key_type<T: BridgeFieldWrite + ?Sized>(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
        value: &T,
    ) -> Result<Oop> {
        self.put_with_key_type(key, key_type, value.to_bridge_field_value())
    }

    pub fn put_string(&mut self, key: &str, value: impl AsRef<str>) -> Result<Oop> {
        self.put_string_with_key_type(key, BridgeKeyType::String, value)
    }

    pub fn put_string_with_key_type(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
        value: impl AsRef<str>,
    ) -> Result<Oop> {
        self.put_with_key_type(
            key,
            key_type,
            BridgeValue::String(value.as_ref().to_string()),
        )
    }

    pub fn put_symbol(&mut self, key: &str, value: impl AsRef<str>) -> Result<Oop> {
        self.put_symbol_with_key_type(key, BridgeKeyType::String, value)
    }

    pub fn put_symbol_with_key_type(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
        value: impl AsRef<str>,
    ) -> Result<Oop> {
        self.put_with_key_type(
            key,
            key_type,
            BridgeValue::Symbol(value.as_ref().to_string()),
        )
    }

    pub fn put_smallint(&mut self, key: &str, value: i64) -> Result<Oop> {
        self.put_smallint_with_key_type(key, BridgeKeyType::String, value)
    }

    pub fn put_smallint_with_key_type(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
        value: i64,
    ) -> Result<Oop> {
        self.put_with_key_type(key, key_type, BridgeValue::SmallInt(value))
    }

    pub fn put_bool(&mut self, key: &str, value: bool) -> Result<Oop> {
        self.put_bool_with_key_type(key, BridgeKeyType::String, value)
    }

    pub fn put_bool_with_key_type(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
        value: bool,
    ) -> Result<Oop> {
        self.put_with_key_type(key, key_type, BridgeValue::Bool(value))
    }

    pub fn put_vec<T: BridgeFieldWrite>(&mut self, key: &str, values: &[T]) -> Result<Oop> {
        self.put_vec_with_key_type(key, BridgeKeyType::String, values)
    }

    pub fn put_vec_with_key_type<T: BridgeFieldWrite>(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
        values: &[T],
    ) -> Result<Oop> {
        self.put_with_key_type(
            key,
            key_type,
            BridgeValue::array(values.iter().map(BridgeFieldWrite::to_bridge_field_value)),
        )
    }

    pub fn put_map<T: BridgeFieldWrite>(
        &mut self,
        key: &str,
        values: &BTreeMap<String, T>,
    ) -> Result<Oop> {
        self.put_map_with_key_type(key, BridgeKeyType::String, values)
    }

    pub fn put_map_with_key_type<T: BridgeFieldWrite>(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
        values: &BTreeMap<String, T>,
    ) -> Result<Oop> {
        self.put_with_key_type(
            key,
            key_type,
            BridgeValue::dictionary(
                values
                    .iter()
                    .map(|(entry_key, value)| (entry_key.clone(), value.to_bridge_field_value())),
            ),
        )
    }

    pub fn put_optional<T: BridgeFieldWrite>(
        &mut self,
        key: &str,
        value: &Option<T>,
    ) -> Result<Oop> {
        self.put_optional_with_key_type(key, BridgeKeyType::String, value)
    }

    pub fn put_optional_with_key_type<T: BridgeFieldWrite>(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
        value: &Option<T>,
    ) -> Result<Oop> {
        self.put_with_key_type(key, key_type, value.to_bridge_field_value())
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

    pub fn get_bridge_value(&mut self, key: &str) -> Result<BridgeValue> {
        self.get_bridge_value_with_key_type(key, BridgeKeyType::String)
    }

    pub fn get_bridge_value_with_key_type(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
    ) -> Result<BridgeValue> {
        self.get_field_with_key_type(key, key_type)
    }

    pub fn get_bridge_value_with_depth(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
        max_depth: usize,
    ) -> Result<BridgeValue> {
        let oop = self.get_oop_with_key_type(key, key_type)?;
        BridgeValue::from_oop_with_depth(self.session, oop, max_depth)
    }

    pub fn get_string(&mut self, key: &str) -> Result<String> {
        self.get_string_with_key_type(key, BridgeKeyType::String)
    }

    pub fn get_string_with_key_type(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
    ) -> Result<String> {
        let oop = self.get_oop_with_key_type(key, key_type)?;
        self.session.fetch_string(oop)
    }

    pub fn get_smallint(&mut self, key: &str) -> Result<i64> {
        self.get_smallint_with_key_type(key, BridgeKeyType::String)
    }

    pub fn get_smallint_with_key_type(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
    ) -> Result<i64> {
        match self.get_value_with_key_type(key, key_type)? {
            Value::SmallInt(value) => Ok(value),
            other => Err(unexpected_field(key, "SmallInt", other)),
        }
    }

    pub fn get_bool(&mut self, key: &str) -> Result<bool> {
        self.get_bool_with_key_type(key, BridgeKeyType::String)
    }

    pub fn get_bool_with_key_type(&mut self, key: &str, key_type: BridgeKeyType) -> Result<bool> {
        match self.get_value_with_key_type(key, key_type)? {
            Value::Bool(value) => Ok(value),
            other => Err(unexpected_field(key, "Bool", other)),
        }
    }

    pub fn get_dictionary(&mut self, key: &str) -> Result<BridgeDictionary<'_>> {
        self.get_dictionary_with_key_type(key, BridgeKeyType::String)
    }

    pub fn get_dictionary_with_key_type(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
    ) -> Result<BridgeDictionary<'_>> {
        let oop = self.get_oop_with_key_type(key, key_type)?;
        Ok(BridgeDictionary::from_oop(self.session, oop))
    }

    pub fn get_field<T: BridgeFieldRead>(&mut self, key: &str) -> Result<T> {
        self.get_field_with_key_type(key, BridgeKeyType::String)
    }

    pub fn get_field_with_key_type<T: BridgeFieldRead>(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
    ) -> Result<T> {
        let mut dictionary = BridgeDictionary::from_oop(self.session, self.oop);
        BridgeFieldRead::read_bridge_field(&mut dictionary, key, key_type)
    }

    pub fn get_mapped<T: BridgeMapped>(&mut self, key: &str) -> Result<T> {
        self.get_field(key)
    }

    pub fn get_mapped_with_key_type<T: BridgeMapped>(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
    ) -> Result<T> {
        self.get_field_with_key_type(key, key_type)
    }

    pub fn get_vec<T: BridgeFieldRead>(&mut self, key: &str) -> Result<Vec<T>> {
        self.get_field(key)
    }

    pub fn get_vec_with_key_type<T: BridgeFieldRead>(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
    ) -> Result<Vec<T>> {
        self.get_field_with_key_type(key, key_type)
    }

    pub fn get_map<T: BridgeFieldRead>(&mut self, key: &str) -> Result<BTreeMap<String, T>> {
        self.get_field(key)
    }

    pub fn get_map_with_key_type<T: BridgeFieldRead>(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
    ) -> Result<BTreeMap<String, T>> {
        self.get_field_with_key_type(key, key_type)
    }

    pub fn get_optional<T: BridgeFieldRead>(&mut self, key: &str) -> Result<Option<T>> {
        self.get_field(key)
    }

    pub fn get_optional_with_key_type<T: BridgeFieldRead>(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
    ) -> Result<Option<T>> {
        self.get_field_with_key_type(key, key_type)
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

    pub fn keys(&mut self) -> Result<Vec<BridgeKeySummary>> {
        dictionary_keys(self.session, self.oop)
    }

    pub fn contains_key(&mut self, key: &str) -> Result<bool> {
        self.contains_key_with_key_type(key, BridgeKeyType::String)
    }

    pub fn contains_key_with_key_type(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
    ) -> Result<bool> {
        dictionary_contains_key(self.session, self.oop, key, key_type)
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

    pub fn put_field<T: BridgeFieldWrite + ?Sized>(&mut self, key: &str, value: &T) -> Result<Oop> {
        self.put_field_with_key_type(key, BridgeKeyType::String, value)
    }

    pub fn put_field_with_key_type<T: BridgeFieldWrite + ?Sized>(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
        value: &T,
    ) -> Result<Oop> {
        self.put_with_key_type(key, key_type, value.to_bridge_field_value())
    }

    pub fn put_string(&mut self, key: &str, value: impl AsRef<str>) -> Result<Oop> {
        self.put_string_with_key_type(key, BridgeKeyType::String, value)
    }

    pub fn put_string_with_key_type(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
        value: impl AsRef<str>,
    ) -> Result<Oop> {
        self.put_with_key_type(
            key,
            key_type,
            BridgeValue::String(value.as_ref().to_string()),
        )
    }

    pub fn put_symbol(&mut self, key: &str, value: impl AsRef<str>) -> Result<Oop> {
        self.put_symbol_with_key_type(key, BridgeKeyType::String, value)
    }

    pub fn put_symbol_with_key_type(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
        value: impl AsRef<str>,
    ) -> Result<Oop> {
        self.put_with_key_type(
            key,
            key_type,
            BridgeValue::Symbol(value.as_ref().to_string()),
        )
    }

    pub fn put_smallint(&mut self, key: &str, value: i64) -> Result<Oop> {
        self.put_smallint_with_key_type(key, BridgeKeyType::String, value)
    }

    pub fn put_smallint_with_key_type(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
        value: i64,
    ) -> Result<Oop> {
        self.put_with_key_type(key, key_type, BridgeValue::SmallInt(value))
    }

    pub fn put_bool(&mut self, key: &str, value: bool) -> Result<Oop> {
        self.put_bool_with_key_type(key, BridgeKeyType::String, value)
    }

    pub fn put_bool_with_key_type(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
        value: bool,
    ) -> Result<Oop> {
        self.put_with_key_type(key, key_type, BridgeValue::Bool(value))
    }

    pub fn put_vec<T: BridgeFieldWrite>(&mut self, key: &str, values: &[T]) -> Result<Oop> {
        self.put_vec_with_key_type(key, BridgeKeyType::String, values)
    }

    pub fn put_vec_with_key_type<T: BridgeFieldWrite>(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
        values: &[T],
    ) -> Result<Oop> {
        self.put_with_key_type(
            key,
            key_type,
            BridgeValue::array(values.iter().map(BridgeFieldWrite::to_bridge_field_value)),
        )
    }

    pub fn put_map<T: BridgeFieldWrite>(
        &mut self,
        key: &str,
        values: &BTreeMap<String, T>,
    ) -> Result<Oop> {
        self.put_map_with_key_type(key, BridgeKeyType::String, values)
    }

    pub fn put_map_with_key_type<T: BridgeFieldWrite>(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
        values: &BTreeMap<String, T>,
    ) -> Result<Oop> {
        self.put_with_key_type(
            key,
            key_type,
            BridgeValue::dictionary(
                values
                    .iter()
                    .map(|(entry_key, value)| (entry_key.clone(), value.to_bridge_field_value())),
            ),
        )
    }

    pub fn put_optional<T: BridgeFieldWrite>(
        &mut self,
        key: &str,
        value: &Option<T>,
    ) -> Result<Oop> {
        self.put_optional_with_key_type(key, BridgeKeyType::String, value)
    }

    pub fn put_optional_with_key_type<T: BridgeFieldWrite>(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
        value: &Option<T>,
    ) -> Result<Oop> {
        self.put_with_key_type(key, key_type, value.to_bridge_field_value())
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

    pub fn at_bridge_value(&mut self, key: &str) -> Result<BridgeValue> {
        self.at_bridge_value_with_key_type(key, BridgeKeyType::String)
    }

    pub fn at_bridge_value_with_key_type(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
    ) -> Result<BridgeValue> {
        self.at_field_with_key_type(key, key_type)
    }

    pub fn at_bridge_value_with_depth(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
        max_depth: usize,
    ) -> Result<BridgeValue> {
        let oop = self.at_oop_with_key_type(key, key_type)?;
        BridgeValue::from_oop_with_depth(self.session, oop, max_depth)
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
        self.at_field_with_key_type(key, key_type)
    }

    pub fn at_vec<T: BridgeFieldRead>(&mut self, key: &str) -> Result<Vec<T>> {
        self.at_vec_with_key_type(key, BridgeKeyType::String)
    }

    pub fn at_vec_with_key_type<T: BridgeFieldRead>(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
    ) -> Result<Vec<T>> {
        self.at_field_with_key_type(key, key_type)
    }

    pub fn at_map<T: BridgeFieldRead>(&mut self, key: &str) -> Result<BTreeMap<String, T>> {
        self.at_map_with_key_type(key, BridgeKeyType::String)
    }

    pub fn at_map_with_key_type<T: BridgeFieldRead>(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
    ) -> Result<BTreeMap<String, T>> {
        self.at_field_with_key_type(key, key_type)
    }

    pub fn at_optional<T: BridgeFieldRead>(&mut self, key: &str) -> Result<Option<T>> {
        self.at_optional_with_key_type(key, BridgeKeyType::String)
    }

    pub fn at_optional_with_key_type<T: BridgeFieldRead>(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
    ) -> Result<Option<T>> {
        self.at_field_with_key_type(key, key_type)
    }

    pub fn at_field<T: BridgeFieldRead>(&mut self, key: &str) -> Result<T> {
        self.at_field_with_key_type(key, BridgeKeyType::String)
    }

    pub fn at_field_with_key_type<T: BridgeFieldRead>(
        &mut self,
        key: &str,
        key_type: BridgeKeyType,
    ) -> Result<T> {
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

    fn lookup_error(&self, err: Error) -> Error {
        match err {
            Error::Mapping { .. } => self.nested_error(err),
            other => self.unexpected(format!("lookup failed: {other}")),
        }
    }

    fn read_error(&self, err: Error) -> Error {
        match err {
            Error::Mapping { .. } => self.nested_error(err),
            other => self.unexpected(format!("read failed: {other}")),
        }
    }

    fn nested_error(&self, err: Error) -> Error {
        match err {
            Error::Mapping {
                field,
                expected,
                actual,
            } => Error::Mapping {
                field: self.child_field(&field),
                expected,
                actual,
            },
            other => other,
        }
    }

    fn child_field(&self, field: &str) -> String {
        let field = field
            .split_once(" (")
            .map(|(field, _)| field)
            .unwrap_or(field);
        if field.is_empty() {
            self.key.clone()
        } else {
            format!("{}.{}", self.key, field)
        }
    }

    fn index(&self, index: i64, expected: &'static str) -> Self {
        Self {
            key: format!("{}[{index}]", self.key),
            key_type: self.key_type,
            expected,
        }
    }

    fn map_key(&self, key: &str, expected: &'static str) -> Self {
        Self {
            key: format!("{}[{}]", self.key, quoted_path_key(key)),
            key_type: self.key_type,
            expected,
        }
    }
}

fn quoted_path_key(key: &str) -> String {
    let mut out = String::from("\"");
    for ch in key.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

fn unexpected_field(key: &str, expected: &'static str, actual: Value) -> Error {
    Error::Mapping {
        field: key.to_string(),
        expected,
        actual: format!("{actual:?}"),
    }
}

fn dictionary_contains_key(
    session: &mut Session,
    dictionary: Oop,
    key: &str,
    key_type: BridgeKeyType,
) -> Result<bool> {
    let key_oop = BridgeKey::new(key, key_type).to_oop(session)?;
    match session.perform(dictionary, "includesKey:", &[key_oop])? {
        Value::Bool(value) => Ok(value),
        other => Err(unexpected_field(key, "Bool", other)),
    }
}

fn dictionary_keys(session: &mut Session, dictionary: Oop) -> Result<Vec<BridgeKeySummary>> {
    let keys = session.perform_oop(dictionary, "keys", &[])?;
    let array = session.perform_oop(keys, "asArray", &[])?;
    let size = session.fetch_size(array)?;
    if size < 0 {
        return Err(Error::NegativeSize(size));
    }

    let mut summaries = Vec::with_capacity(size as usize);
    for index in 1..=size {
        let index_oop = session.smallint_oop(index);
        let oop = session.perform_oop(array, "at:", &[index_oop])?;
        let class_oop = session.fetch_class(oop)?;
        let printed = session.perform_oop(oop, "printString", &[])?;
        let print_string = session.fetch_string(printed)?;
        let identity_id = session.identity_for_oop(oop);
        summaries.push(BridgeKeySummary {
            oop,
            class_oop,
            print_string,
            identity_id,
        });
    }
    Ok(summaries)
}

fn dictionary_string_entries<T: BridgeFieldRead>(
    session: &mut Session,
    dictionary: Oop,
    context: &BridgeFieldContext,
) -> Result<BTreeMap<String, T>> {
    let keys = dictionary_keys(session, dictionary)
        .map_err(|err| context.unexpected(format!("keys lookup failed: {err}")))?;
    let mut entries = BTreeMap::new();
    for summary in keys {
        let key = session.fetch_string(summary.oop).map_err(|err| {
            context.unexpected(format!(
                "non-string dictionary key {} ({err})",
                summary.print_string
            ))
        })?;
        let value_oop = session
            .perform_oop(dictionary, "at:", &[summary.oop])
            .map_err(|err| context.map_key(&key, T::expected_type()).lookup_error(err))?;
        let entry_context = context.map_key(&key, T::expected_type());
        let value = T::read_bridge_oop(session, value_oop, &entry_context)?;
        entries.insert(key, value);
    }
    Ok(entries)
}

fn bridge_value_dictionary_from_oop(
    session: &mut Session,
    dictionary: Oop,
    remaining_depth: usize,
    seen: &mut BTreeSet<u64>,
) -> Result<BridgeValue> {
    let keys = dictionary_keys(session, dictionary)?;
    let mut string_entries = BTreeMap::new();
    let mut keyed_entries = Vec::new();
    let mut only_string_keys = true;

    for summary in keys {
        let key = bridge_key_from_summary(session, &summary)?;
        let value_oop = session.perform_oop(dictionary, "at:", &[summary.oop])?;
        let value = BridgeValue::from_oop_inner(
            session,
            value_oop,
            remaining_depth.saturating_sub(1),
            seen,
        )?;
        if key.key_type == BridgeKeyType::String {
            string_entries.insert(key.name.clone(), value.clone());
        } else {
            only_string_keys = false;
        }
        keyed_entries.push((key, value));
    }

    if only_string_keys {
        Ok(BridgeValue::Dictionary(string_entries))
    } else {
        Ok(BridgeValue::KeyedDictionary(keyed_entries))
    }
}

fn bridge_key_from_summary(session: &mut Session, summary: &BridgeKeySummary) -> Result<BridgeKey> {
    if session.is_kind_of(summary.oop, "Symbol")? {
        return Ok(BridgeKey::symbol(session.fetch_string(summary.oop)?));
    }
    if session.is_kind_of(summary.oop, "String")? {
        return Ok(BridgeKey::string(session.fetch_string(summary.oop)?));
    }
    Err(Error::Mapping {
        field: "dictionary key".to_string(),
        expected: "String or Symbol",
        actual: format!("{} (OOP {})", summary.print_string, summary.oop.raw()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_array_context_reports_element_index_and_expected_type() {
        let context = BridgeFieldContext::new("tags", BridgeKeyType::String, "Array");
        let err = context
            .index(2, "String")
            .unexpected("OOP 1234".to_string())
            .to_string();

        assert!(err.contains("tags[2]"));
        assert!(err.contains("String"));
        assert!(err.contains("OOP 1234"));
    }

    #[test]
    fn nested_mapping_context_reports_full_field_path() {
        let context =
            BridgeFieldContext::new("booking.customer", BridgeKeyType::String, "Dictionary");
        let err = context.nested_error(Error::Mapping {
            field: "name (Symbol)".to_string(),
            expected: "String",
            actual: "OOP 1234".to_string(),
        });

        assert_eq!(
            err.to_string(),
            "field booking.customer.name expected GemStone value type String, got OOP 1234"
        );
    }

    #[test]
    fn lookup_context_reports_missing_key_path() {
        let context = BridgeFieldContext::new("booking.items", BridgeKeyType::String, "Array");
        let err = context.lookup_error(Error::GemStone {
            number: 2010,
            fatal: false,
            message: "key not found".to_string(),
        });

        assert_eq!(
            err.to_string(),
            "field booking.items (String) expected GemStone value type Array, got lookup failed: GemStone error #2010 fatal=false: key not found"
        );
    }

    #[test]
    fn nested_array_lookup_reports_index_path() {
        let context = BridgeFieldContext::new("booking.items", BridgeKeyType::String, "Array");
        let err = context.index(2, "Customer").lookup_error(Error::GemStone {
            number: 2011,
            fatal: false,
            message: "index out of bounds".to_string(),
        });

        assert_eq!(
            err.to_string(),
            "field booking.items[2] (String) expected GemStone value type Customer, got lookup failed: GemStone error #2011 fatal=false: index out of bounds"
        );
    }

    #[test]
    fn optional_fields_write_some_or_nil() {
        let none: Option<String> = None;
        assert_eq!(none.to_bridge_field_value(), BridgeValue::Nil);

        let some = Some("reference".to_string());
        assert_eq!(
            some.to_bridge_field_value(),
            BridgeValue::String("reference".to_string())
        );
    }

    #[test]
    fn bridge_value_from_collections_uses_field_conversions() {
        let values = BridgeValue::from(vec!["priority".to_string(), "demo".to_string()]);
        let BridgeValue::Array(values) = values else {
            panic!("expected array bridge value");
        };
        assert_eq!(values[0], BridgeValue::String("priority".to_string()));
        assert_eq!(values[1], BridgeValue::String("demo".to_string()));

        let labels = BTreeMap::from([("source".to_string(), "rust".to_string())]);
        let BridgeValue::Dictionary(entries) = BridgeValue::from(labels) else {
            panic!("expected dictionary bridge value");
        };
        assert_eq!(entries["source"], BridgeValue::String("rust".to_string()));

        let none: BridgeValue = Option::<String>::None.into();
        assert_eq!(none, BridgeValue::Nil);
    }

    #[test]
    fn string_slices_write_string_fields() {
        assert_eq!(
            "hello".to_bridge_field_value(),
            BridgeValue::String("hello".to_string())
        );
    }

    #[test]
    fn string_keyed_maps_write_dictionary_values() {
        let mut labels = BTreeMap::new();
        labels.insert("source".to_string(), "rust".to_string());
        labels.insert("priority".to_string(), "high".to_string());

        let BridgeValue::Dictionary(entries) = labels.to_bridge_field_value() else {
            panic!("expected dictionary bridge value");
        };
        assert_eq!(entries["source"], BridgeValue::String("rust".to_string()));
        assert_eq!(entries["priority"], BridgeValue::String("high".to_string()));
    }

    #[test]
    fn map_context_reports_quoted_string_key() {
        let context = BridgeFieldContext::new("labels", BridgeKeyType::String, "Dictionary");
        let err = context
            .map_key("source name", "String")
            .unexpected("OOP 1234".to_string())
            .to_string();

        assert!(err.contains("labels[\"source name\"]"));
        assert!(err.contains("String"));
    }
}
