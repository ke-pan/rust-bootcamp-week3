mod serialize;

use std::{
    collections::{BTreeMap, BTreeSet},
    ops::{Deref, DerefMut},
};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
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

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SimpleError {
    value: String,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Integer {
    value: i64,
}

impl Integer {
    #[allow(dead_code)]
    pub fn new(value: i64) -> Self {
        Integer { value }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BulkString {
    value: String,
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

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Null {}

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
