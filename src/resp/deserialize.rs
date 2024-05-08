use super::*;
use bytes::{Buf, Bytes};
use std::str::from_utf8;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RespDeserializeError {
    #[error("Unknown RESP Type")]
    UnknownRespType,
    #[error("Not Complete")]
    NotComplete,
    #[error("Wrong Format")]
    WrongFormat,
    #[error("UTF-8 Error")]
    Utf8Error(#[from] std::str::Utf8Error),
}

impl TryFrom<Vec<u8>> for Resp {
    type Error = RespDeserializeError;

    fn try_from(buf: Vec<u8>) -> Result<Resp, RespDeserializeError> {
        let mut buf = buf.into();
        let resp = _try_from(&mut buf)?;
        if !buf.is_empty() {
            return Err(RespDeserializeError::WrongFormat);
        }
        Ok(resp)
    }
}

fn _try_from(buf: &mut Bytes) -> Result<Resp, RespDeserializeError> {
    if buf.len() < 3 {
        return Err(RespDeserializeError::NotComplete);
    }

    match buf[0] {
        b'+' => {
            buf.advance(1);
            let s = deserialize_simple_string(buf)?;
            Ok(Resp::SimpleString(s))
        }
        b'-' => {
            buf.advance(1);
            let s = deserialize_simple_error(buf)?;
            Ok(Resp::SimpleError(s))
        }
        b'%' => {
            buf.advance(1);
            let m = deserialize_map(buf)?;
            Ok(Resp::Map(Box::new(m)))
        }
        b':' => {
            buf.advance(1);
            let i = deserialize_integer(buf)?;
            Ok(Resp::Integer(i))
        }
        b'$' => {
            buf.advance(1);
            let b = deserialize_bulk_string(buf)?;
            Ok(Resp::BulkString(b))
        }
        b'_' => {
            buf.advance(1);
            let n = deserialize_null(buf)?;
            Ok(Resp::Null(n))
        }
        b'#' => {
            buf.advance(1);
            let b = deserialize_boolean(buf)?;
            Ok(Resp::Boolean(b))
        }
        b',' => {
            buf.advance(1);
            let d = deserialize_double(buf)?;
            Ok(Resp::Double(d))
        }
        b'!' => {
            buf.advance(1);
            let b = deserialize_bulk_error(buf)?;
            Ok(Resp::BulkError(BulkError::new(b.value)))
        }
        b'*' => {
            buf.advance(1);
            let a = deserialize_array(buf)?;
            Ok(Resp::Array(a))
        }
        b'~' => {
            buf.advance(1);
            let s = deserialize_set(buf)?;
            Ok(Resp::Set(s))
        }
        _ => Err(RespDeserializeError::UnknownRespType),
    }
}

trait Deserialize {
    fn deserialize<'a>(&'a mut self, buf: &'a [u8]) -> Result<&[u8], RespDeserializeError>;
}

fn deserialize_simple_string(buf: &mut Bytes) -> Result<SimpleString, RespDeserializeError> {
    let bytes = find_crlf(buf)?;
    match from_utf8(bytes.as_ref()) {
        Ok(s) => Ok(SimpleString::new(s)),
        Err(e) => Err(RespDeserializeError::Utf8Error(e)),
    }
}

fn deserialize_simple_error(buf: &mut Bytes) -> Result<SimpleError, RespDeserializeError> {
    let bytes = find_crlf(buf)?;
    match from_utf8(bytes.as_ref()) {
        Ok(s) => Ok(SimpleError::new(s)),
        Err(e) => Err(RespDeserializeError::Utf8Error(e)),
    }
}

fn deserialize_map(buf: &mut Bytes) -> Result<Map, RespDeserializeError> {
    let bytes = find_crlf(buf)?;
    let len = match from_utf8(bytes.as_ref()) {
        Ok(s) => s
            .parse::<usize>()
            .map_err(|_| RespDeserializeError::WrongFormat)?,
        Err(e) => return Err(RespDeserializeError::Utf8Error(e)),
    };
    let mut map = Map::default();
    for _ in 0..len {
        let key = _try_from(buf)?
            .try_into()
            .map_err(|_| RespDeserializeError::WrongFormat)?;
        let value = _try_from(buf)?;
        map.insert(key, value);
    }
    Ok(map)
}

fn deserialize_integer(buf: &mut Bytes) -> Result<Integer, RespDeserializeError> {
    let bytes = find_crlf(buf)?;
    match from_utf8(bytes.as_ref()) {
        Ok(s) => Ok(Integer::new(
            s.parse::<i64>()
                .map_err(|_| RespDeserializeError::WrongFormat)?,
        )),
        Err(e) => Err(RespDeserializeError::Utf8Error(e)),
    }
}

fn deserialize_bulk_string(buf: &mut Bytes) -> Result<BulkString, RespDeserializeError> {
    let bytes = find_crlf(buf)?;
    let len = match from_utf8(bytes.as_ref()) {
        Ok(s) => s
            .parse::<usize>()
            .map_err(|_| RespDeserializeError::WrongFormat)?,
        Err(e) => return Err(RespDeserializeError::Utf8Error(e)),
    };
    if buf.len() < len + 2 {
        return Err(RespDeserializeError::NotComplete);
    }
    let res = buf.split_to(len);
    if buf[0] != b'\r' || buf[1] != b'\n' {
        return Err(RespDeserializeError::WrongFormat);
    }
    buf.advance(2);
    match from_utf8(res.as_ref()) {
        Ok(s) => Ok(BulkString::new(s)),
        Err(e) => Err(RespDeserializeError::Utf8Error(e)),
    }
}

fn deserialize_null(buf: &mut Bytes) -> Result<Null, RespDeserializeError> {
    if buf.len() < 2 {
        return Err(RespDeserializeError::NotComplete);
    }
    if buf[0] != b'\r' || buf[1] != b'\n' {
        return Err(RespDeserializeError::WrongFormat);
    }
    buf.advance(2);
    Ok(Null {})
}

fn deserialize_boolean(buf: &mut Bytes) -> Result<Boolean, RespDeserializeError> {
    let bytes = find_crlf(buf)?;
    if bytes.len() != 1 {
        return Err(RespDeserializeError::WrongFormat);
    }
    match bytes[0] as char {
        't' => Ok(Boolean::new(true)),
        'f' => Ok(Boolean::new(false)),
        _ => Err(RespDeserializeError::WrongFormat),
    }
}

fn deserialize_double(buf: &mut Bytes) -> Result<Double, RespDeserializeError> {
    let bytes = find_crlf(buf)?;
    match from_utf8(bytes.as_ref()) {
        Ok(s) => Ok(Double::new(
            s.parse::<f64>()
                .map_err(|_| RespDeserializeError::WrongFormat)?,
        )),
        Err(e) => Err(RespDeserializeError::Utf8Error(e)),
    }
}

fn deserialize_bulk_error(buf: &mut Bytes) -> Result<BulkError, RespDeserializeError> {
    let bytes = find_crlf(buf)?;
    let len = match from_utf8(bytes.as_ref()) {
        Ok(s) => s
            .parse::<usize>()
            .map_err(|_| RespDeserializeError::WrongFormat)?,
        Err(e) => return Err(RespDeserializeError::Utf8Error(e)),
    };
    if buf.len() < len + 2 {
        return Err(RespDeserializeError::NotComplete);
    }
    let res = buf.split_to(len);
    if buf[0] != b'\r' || buf[1] != b'\n' {
        return Err(RespDeserializeError::WrongFormat);
    }
    buf.advance(2);
    match from_utf8(res.as_ref()) {
        Ok(s) => Ok(BulkError::new(s)),
        Err(e) => Err(RespDeserializeError::Utf8Error(e)),
    }
}

fn deserialize_array(buf: &mut Bytes) -> Result<Array, RespDeserializeError> {
    let bytes = find_crlf(buf)?;
    let len = match from_utf8(bytes.as_ref()) {
        Ok(s) => s
            .parse::<usize>()
            .map_err(|_| RespDeserializeError::WrongFormat)?,
        Err(e) => return Err(RespDeserializeError::Utf8Error(e)),
    };
    let mut array = Array::default();
    for _ in 0..len {
        let value = _try_from(buf)?;
        array.push(value);
    }
    Ok(array)
}

fn deserialize_set(buf: &mut Bytes) -> Result<Set, RespDeserializeError> {
    let bytes = find_crlf(buf)?;
    let len = match from_utf8(bytes.as_ref()) {
        Ok(s) => s
            .parse::<usize>()
            .map_err(|_| RespDeserializeError::WrongFormat)?,
        Err(e) => return Err(RespDeserializeError::Utf8Error(e)),
    };
    let mut set = Set::default();
    for _ in 0..len {
        let value = _try_from(buf)?;
        set.insert(
            value
                .try_into()
                .map_err(|_| RespDeserializeError::WrongFormat)?,
        );
    }
    Ok(set)
}

fn find_crlf(buf: &mut Bytes) -> Result<Bytes, RespDeserializeError> {
    let i = buf
        .iter()
        .position(|&c| c == b'\r')
        .ok_or(RespDeserializeError::NotComplete)?;
    if i + 1 >= buf.len() {
        return Err(RespDeserializeError::NotComplete);
    }
    if buf[i + 1] != b'\n' {
        return Err(RespDeserializeError::WrongFormat);
    }
    let res = buf.split_to(i);
    buf.advance(2);
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_simple_string() {
        let buf: &[u8] = b"+OK\r\n";
        let r = Resp::try_from(buf.to_vec()).unwrap();
        assert_eq!(r, Resp::SimpleString(SimpleString::new("OK")));

        let buf: &[u8] = b"+OK\r";
        let r = Resp::try_from(buf.to_vec());
        assert!(r.is_err());

        let buf: &[u8] = b"+OK\r\n+OK\r\n";
        let r = Resp::try_from(buf.to_vec());
        assert!(r.is_err());
    }

    #[test]
    fn test_deserialize_simple_error() {
        let buf: &[u8] = b"-ERR\r\n";
        let r = Resp::try_from(buf.to_vec()).unwrap();
        assert_eq!(r, Resp::SimpleError(SimpleError::new("ERR")));

        let buf = b"-ERR\r";
        let r = Resp::try_from(buf.to_vec());
        assert!(r.is_err());

        let buf = b"-ERR\r\n+OK\r\n";
        let r = Resp::try_from(buf.to_vec());
        assert!(r.is_err());
    }

    #[test]
    fn test_deserialize_map() {
        let buf = b"%2\r\n+value1\r\n!5\r\nerror\r\n#t\r\n*0\r\n";
        let r: Resp = buf.to_vec().try_into().unwrap();
        let mut m = Map::default();
        m.insert(
            Key::SimpleString(SimpleString::new("value1")),
            Resp::BulkError(BulkError::new("error")),
        );
        m.insert(
            Key::Boolean(Boolean::new(true)),
            Resp::Array(Array::default()),
        );
        assert_eq!(r, Resp::Map(Box::new(m)))
    }

    #[test]
    fn test_deserialize_integer() {
        let buf = b":123\r\n";
        let r: Resp = buf.to_vec().try_into().unwrap();
        assert_eq!(r, Resp::Integer(Integer::new(123)));

        let buf = b":123\r";
        let r = Resp::try_from(buf.to_vec());
        assert!(r.is_err());

        let buf = b":123\r\n+OK\r\n";
        let r = Resp::try_from(buf.to_vec());
        assert!(r.is_err());
    }

    #[test]
    fn test_deserialize_bulk_string() {
        let buf = b"$6\r\nfoobar\r\n";
        let r: Resp = buf.to_vec().try_into().unwrap();
        assert_eq!(r, Resp::BulkString(BulkString::new("foobar")));

        let buf = b"$6\r\nfoobar\r";
        let r = Resp::try_from(buf.to_vec());
        assert!(r.is_err());

        let buf = b"$6\r\nfoobar\r\n+OK\r\n";
        let r = Resp::try_from(buf.to_vec());
        assert!(r.is_err());
    }

    #[test]
    fn test_deserialize_null() {
        let buf = b"_\r\n";
        let r: Resp = buf.to_vec().try_into().unwrap();
        assert_eq!(r, Resp::Null(Null {}));

        let buf = b"_\r";
        let r = Resp::try_from(buf.to_vec());
        assert!(r.is_err());

        let buf = b"_\r\n+OK\r\n";
        let r = Resp::try_from(buf.to_vec());
        assert!(r.is_err());
    }

    #[test]
    fn test_deserialize_boolean() {
        let buf = b"#t\r\n";
        let r: Resp = buf.to_vec().try_into().unwrap();
        assert_eq!(r, Resp::Boolean(Boolean::new(true)));

        let buf = b"#f\r\n";
        let r: Resp = buf.to_vec().try_into().unwrap();
        assert_eq!(r, Resp::Boolean(Boolean::new(false)));

        let buf = b"#t\r";
        let r = Resp::try_from(buf.to_vec());
        assert!(r.is_err());

        let buf = b"#t\r\n+OK\r\n";
        let r = Resp::try_from(buf.to_vec());
        assert!(r.is_err());
    }

    #[test]
    fn test_deserialize_double() {
        let buf = b",123.45\r\n";
        let r: Resp = buf.to_vec().try_into().unwrap();
        assert_eq!(r, Resp::Double(Double::new(123.45)));

        let buf = b",+1e9\r\n";
        let r: Resp = buf.to_vec().try_into().unwrap();
        assert_eq!(r, Resp::Double(Double::new(1e9)));

        let buf = b",-1.23e-9\r\n";
        let r: Resp = buf.to_vec().try_into().unwrap();
        assert_eq!(r, Resp::Double(Double::new(-1.23e-9)));

        let buf = b",123.45\r";
        let r = Resp::try_from(buf.to_vec());
        assert!(r.is_err());

        let buf = b",123.45\r\n+OK\r\n";
        let r = Resp::try_from(buf.to_vec());
        assert!(r.is_err());
    }

    #[test]
    fn test_deserialize_bulk_error() {
        let buf = b"!5\r\nerror\r\n";
        let r: Resp = buf.to_vec().try_into().unwrap();
        assert_eq!(r, Resp::BulkError(BulkError::new("error")));

        let buf = b"!5\r\nerror\r";
        let r = Resp::try_from(buf.to_vec());
        assert!(r.is_err());

        let buf = b"!5\r\nerror\r\n+OK\r\n";
        let r = Resp::try_from(buf.to_vec());
        assert!(r.is_err());
    }

    #[test]
    fn test_deserialize_array() {
        let buf = b"*3\r\n+OK\r\n:123\r\n$6\r\nfoobar\r\n";
        let r: Resp = buf.to_vec().try_into().unwrap();
        let mut a = Array::default();
        a.push(Resp::SimpleString(SimpleString::new("OK")));
        a.push(Resp::Integer(Integer::new(123)));
        a.push(Resp::BulkString(BulkString::new("foobar")));
        assert_eq!(r, Resp::Array(a));

        let buf = b"*0\r\n";
        let r: Resp = buf.to_vec().try_into().unwrap();
        let a = Array::default();
        assert_eq!(r, Resp::Array(a));

        let buf = b"*3\r\n+OK\r\n:123\r\n$6\r\nfoobar\r";
        let r = Resp::try_from(buf.to_vec());
        assert!(r.is_err());

        let buf = b"*3\r\n+OK\r\n:123\r\n$6\r\nfoobar\r\n+OK\r\n";
        let r = Resp::try_from(buf.to_vec());
        assert!(r.is_err());
    }

    #[test]
    fn test_deserialize_set() {
        let buf = b"~2\r\n+value1\r\n#f\r\n";
        let r: Resp = buf.to_vec().try_into().unwrap();
        let mut s = Set::default();
        s.insert(Key::SimpleString(SimpleString::new("value1")));
        s.insert(Key::Boolean(Boolean::new(false)));
        assert_eq!(r, Resp::Set(s));

        let buf = b"~0\r\n";
        let r: Resp = buf.to_vec().try_into().unwrap();
        let s = Set::default();
        assert_eq!(r, Resp::Set(s));

        let buf = b"~2\r\n+value1\r\n#f\r";
        let r = Resp::try_from(buf.to_vec());
        assert!(r.is_err());

        let buf = b"~2\r\n+value1\r\n#f\r\n+OK\r\n";
        let r = Resp::try_from(buf.to_vec());
        assert!(r.is_err());
    }
}
