mod deserialize;
mod serialize;

pub use deserialize::RespDeserializeError;
pub use serialize::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    ops::{Deref, DerefMut},
};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[allow(dead_code)]
pub enum Key {
    SimpleString(SimpleString),
    SimpleError(SimpleError),
    Integer(Integer),
    BulkString(BulkString),
    BulkError(BulkError),
    Null(Null),
    Boolean(Boolean),
}

#[derive(Debug, Error)]
#[error("Unsupported Key")]
pub struct UnsupportedKey;

impl TryFrom<Resp> for Key {
    type Error = UnsupportedKey;
    fn try_from(value: Resp) -> Result<Self, Self::Error> {
        match value {
            Resp::Boolean(v) => Ok(Key::Boolean(v)),
            Resp::BulkString(v) => Ok(Key::BulkString(v)),
            Resp::SimpleString(v) => Ok(Key::SimpleString(v)),
            Resp::SimpleError(v) => Ok(Key::SimpleError(v)),
            Resp::BulkError(v) => Ok(Key::BulkError(v)),
            Resp::Integer(v) => Ok(Key::Integer(v)),
            Resp::Null(v) => Ok(Key::Null(v)),
            _ => Err(UnsupportedKey),
        }
    }
}

impl From<Key> for Resp {
    fn from(value: Key) -> Self {
        match value {
            Key::Boolean(v) => Resp::Boolean(v),
            Key::BulkString(v) => Resp::BulkString(v),
            Key::SimpleString(v) => Resp::SimpleString(v),
            Key::SimpleError(v) => Resp::SimpleError(v),
            Key::BulkError(v) => Resp::BulkError(v),
            Key::Integer(v) => Resp::Integer(v),
            Key::Null(v) => Resp::Null(v),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Resp {
    SimpleString(SimpleString),
    SimpleError(SimpleError),
    Integer(Integer),
    BulkString(BulkString),
    Array(Array),
    Null(Null),
    Boolean(Boolean),
    Double(Double),
    BulkError(BulkError),
    Map(Box<Map>),
    Set(Set),
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimpleString {
    value: String,
}

impl SimpleString {
    #[allow(dead_code)]
    pub fn new<T: Into<String>>(value: T) -> Self {
        SimpleString {
            value: value.into(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimpleError {
    value: String,
}

impl SimpleError {
    #[allow(dead_code)]
    pub fn new<T: Into<String>>(value: T) -> Self {
        SimpleError {
            value: value.into(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Integer {
    value: i64,
}

impl Integer {
    #[allow(dead_code)]
    pub fn new(value: i64) -> Self {
        Integer { value }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BulkString {
    pub value: String,
}

impl BulkString {
    #[allow(dead_code)]
    pub fn new<T: Into<String>>(value: T) -> Self {
        BulkString {
            value: value.into(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct Array {
    value: Vec<Resp>,
}

impl Deref for Array {
    type Target = Vec<Resp>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for Array {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Null;

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Boolean {
    value: bool,
}

impl Boolean {
    #[allow(dead_code)]
    pub fn new(value: bool) -> Self {
        Boolean { value }
    }
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct Double {
    value: f64,
}

impl Double {
    #[allow(dead_code)]
    pub fn new(value: f64) -> Self {
        Double { value }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BulkError {
    value: String,
}

impl BulkError {
    #[allow(dead_code)]
    pub fn new<T: Into<String>>(value: T) -> Self {
        BulkError {
            value: value.into(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct Map {
    value: BTreeMap<Key, Resp>,
}

impl Deref for Map {
    type Target = BTreeMap<Key, Resp>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for Map {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct Set {
    value: BTreeSet<Key>,
}

impl Deref for Set {
    type Target = BTreeSet<Key>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for Set {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
