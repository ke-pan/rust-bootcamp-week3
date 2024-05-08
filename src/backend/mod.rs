use crate::{
    cmd::{Command, CommandExecutor},
    resp::{Key, Resp},
};
use anyhow::{Ok, Result};
use dashmap::DashMap;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct Storage {
    storage: Arc<DashMap<Key, Resp>>,
}

impl CommandExecutor for Storage {
    fn execute(&self, cmd: Command) -> Result<Option<Resp>> {
        match cmd {
            Command::Get(get) => Ok(self.get(&get.key)),
            Command::Set(set) => {
                self.set(set.key, set.value);
                Ok(None)
            }
            _ => Ok(None),
        }
    }
}

impl Storage {
    pub fn new() -> Self {
        Self {
            storage: DashMap::new().into(),
        }
    }

    fn get(&self, key: &Key) -> Option<Resp> {
        self.storage.get(key).map(|v| v.value().clone())
    }

    fn set(&self, key: Key, value: Resp) {
        self.storage.insert(key, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cmd::{Get, Set},
        resp::{BulkString, Integer},
    };

    #[test]
    fn test_storage() {
        let storage = Storage::new();
        let key = Key::BulkString(BulkString::new("key"));
        let value = Resp::Integer(Integer::new(42));
        let res = storage.execute(Command::Set(Set {
            key: key.clone(),
            value: value.clone(),
        }));
        assert!(res.is_ok());
        let res = storage.execute(Command::Get(Get { key }));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), Some(value));
    }
}
