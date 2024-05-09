use crate::resp::{Key, Null, Resp, SimpleString};
use anyhow::Result;
use thiserror::Error;
use tracing::info;

#[derive(Debug, Error, PartialEq)]
pub enum CommandError {
    #[error("Unsupported Command {0}")]
    UnsupportedCommand(String),
    #[error("Wrong number of arguments, expected {0}, got {1}")]
    WrongNumberOfArguments(usize, usize),
    #[error("Wrong format")]
    WrongFormat,
    #[error("Unsupported Key")]
    UnsupportedKey,
}

#[derive(Debug, Clone)]
pub enum Command {
    Get(Get),
    Set(Set),
    Echo(Echo),
    Cmd,
}

pub trait CommandExecutor {
    fn execute(&self, cmd: Command) -> Result<Option<Resp>>;
}

impl Command {
    pub fn execute(&self, executor: &dyn CommandExecutor) -> Result<Resp> {
        match self {
            Command::Get(c) => c.execute(executor),
            Command::Set(c) => c.execute(executor),
            Command::Echo(c) => c.execute(executor),
            Command::Cmd => Ok(Resp::SimpleString(SimpleString::new("OK"))),
        }
    }
}

impl TryFrom<Resp> for Command {
    type Error = CommandError;
    fn try_from(value: Resp) -> Result<Self, CommandError> {
        match value {
            Resp::Array(v) => {
                let mut iter = v.iter();
                let cmd = iter.next().ok_or(CommandError::WrongFormat)?;
                match cmd {
                    Resp::BulkString(s) => match s.value.to_uppercase().as_str() {
                        "GET" => {
                            if iter.len() != 1 {
                                return Err(CommandError::WrongNumberOfArguments(1, iter.len()));
                            }
                            let key = iter.next().ok_or(CommandError::WrongFormat)?;
                            Ok(Command::Get(Get {
                                key: key
                                    .clone()
                                    .try_into()
                                    .map_err(|_| CommandError::UnsupportedKey)?,
                            }))
                        }
                        "SET" => {
                            if iter.len() != 2 {
                                return Err(CommandError::WrongNumberOfArguments(2, iter.len()));
                            }
                            let key = iter.next().ok_or(CommandError::WrongFormat)?;
                            let value = iter.next().ok_or(CommandError::WrongFormat)?;
                            Ok(Command::Set(Set {
                                key: key
                                    .clone()
                                    .try_into()
                                    .map_err(|_| CommandError::UnsupportedKey)?,
                                value: value.clone(),
                            }))
                        }
                        "COMMAND" => Ok(Command::Cmd),
                        "ECHO" => {
                            if iter.len() != 1 {
                                return Err(CommandError::WrongNumberOfArguments(1, iter.len()));
                            }
                            let msg = iter.next().ok_or(CommandError::WrongFormat)?;
                            Ok(Command::Echo(Echo { msg: msg.clone() }))
                        }
                        cmd => Err(CommandError::UnsupportedCommand(cmd.to_string())),
                    },
                    _ => Err(CommandError::WrongFormat),
                }
            }
            _ => Err(CommandError::WrongFormat),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Get {
    pub key: Key,
}

impl Get {
    fn execute(&self, executor: &dyn CommandExecutor) -> Result<Resp> {
        let res = executor.execute(Command::Get(self.clone()))?;
        info!("Get {:?} with key {:?}", res, self.key);
        match res {
            Some(v) => Ok(v),
            None => Ok(Resp::Null(Null)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Set {
    pub key: Key,
    pub value: Resp,
}

impl Set {
    fn execute(&self, executor: &dyn CommandExecutor) -> Result<Resp> {
        executor.execute(Command::Set(self.clone()))?;
        Ok(Resp::SimpleString(SimpleString::new("OK")))
    }
}

#[derive(Debug, Clone)]
pub struct Echo {
    pub msg: Resp,
}

impl Echo {
    fn execute(&self, _executor: &dyn CommandExecutor) -> Result<Resp> {
        Ok(self.msg.clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::resp::{Array, BulkString, Integer};

    use super::*;

    #[test]
    fn test_try_from_resp() {
        let mut arr = Array::default();
        arr.push(Resp::BulkString(BulkString::new("GET", false)));
        arr.push(Resp::BulkString(BulkString::new("key", false)));
        let resp = Resp::Array(arr);
        let cmd = Command::try_from(resp).unwrap();
        match cmd {
            Command::Get(Get { key }) => {
                assert_eq!(
                    key,
                    Key::BulkString(BulkString {
                        value: "key".to_string(),
                        is_null: false
                    })
                );
            }
            _ => panic!("Expected Get"),
        }

        let mut arr = Array::default();
        arr.push(Resp::BulkString(BulkString::new("SET", false)));
        arr.push(Resp::BulkString(BulkString::new("key", false)));
        arr.push(Resp::Integer(Integer::new(1)));
        let resp = Resp::Array(arr);
        let cmd = Command::try_from(resp).unwrap();
        match cmd {
            Command::Set(Set { key, value }) => {
                assert_eq!(
                    key,
                    Key::BulkString(BulkString {
                        value: "key".to_string(),
                        is_null: false
                    })
                );
                assert_eq!(value, Resp::Integer(Integer::new(1)));
            }
            _ => panic!("Expected Set"),
        }

        let mut arr = Array::default();
        arr.push(Resp::BulkString(BulkString::new("SET", false)));
        arr.push(Resp::BulkString(BulkString::new("key", false)));
        let resp = Resp::Array(arr);
        let cmd = Command::try_from(resp);
        assert!(cmd.is_err());
        assert_eq!(cmd.unwrap_err(), CommandError::WrongNumberOfArguments(2, 1));

        let resp = Resp::BulkString(BulkString::new("SET", false));
        let cmd = Command::try_from(resp);
        assert!(cmd.is_err());
        assert_eq!(cmd.unwrap_err(), CommandError::WrongFormat);
    }

    #[test]
    fn test_parse_echo() {
        let mut arr = Array::default();
        arr.push(Resp::BulkString(BulkString::new("ECHO", false)));
        arr.push(Resp::BulkString(BulkString::new("Hello World", false)));
        let resp = Resp::Array(arr);
        let cmd = Command::try_from(resp).unwrap();
        match cmd {
            Command::Echo(Echo { msg }) => {
                assert_eq!(
                    msg,
                    Resp::BulkString(BulkString {
                        value: "Hello World".to_string(),
                        is_null: false
                    })
                );
            }
            _ => panic!("Expected ECHO"),
        }
    }
}
