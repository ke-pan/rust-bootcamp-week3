use super::*;

pub trait Serialize {
    fn serialize(&self) -> Vec<u8>;
}

impl Serialize for Key {
    fn serialize(&self) -> Vec<u8> {
        match self {
            Key::SimpleString(s) => s.serialize(),
            Key::SimpleError(s) => s.serialize(),
            Key::Integer(s) => s.serialize(),
            Key::BulkString(s) => s.serialize(),
            Key::Null(s) => s.serialize(),
            Key::Boolean(s) => s.serialize(),
            Key::BulkError(s) => s.serialize(),
        }
    }
}

impl Serialize for Resp {
    fn serialize(&self) -> Vec<u8> {
        match self {
            Resp::SimpleString(s) => s.serialize(),
            Resp::SimpleError(s) => s.serialize(),
            Resp::Integer(s) => s.serialize(),
            Resp::BulkString(s) => s.serialize(),
            Resp::Array(s) => s.serialize(),
            Resp::Null(s) => s.serialize(),
            Resp::Boolean(s) => s.serialize(),
            Resp::Double(s) => s.serialize(),
            Resp::BulkError(s) => s.serialize(),
            Resp::Map(s) => s.serialize(),
            // Resp::Set(s) => s.serialize(),
            _ => Vec::new(),
        }
    }
}

impl Serialize for SimpleString {
    fn serialize(&self) -> Vec<u8> {
        format!("+{}\r\n", self.value).as_bytes().to_vec()
    }
}

impl Serialize for SimpleError {
    fn serialize(&self) -> Vec<u8> {
        format!("-{}\r\n", self.value).as_bytes().to_vec()
    }
}

impl Serialize for Integer {
    fn serialize(&self) -> Vec<u8> {
        format!(":{}\r\n", self.value).as_bytes().to_vec()
    }
}

impl Serialize for BulkString {
    fn serialize(&self) -> Vec<u8> {
        format!("${}\r\n{}\r\n", self.value.len(), self.value)
            .as_bytes()
            .to_vec()
    }
}

impl Serialize for Null {
    fn serialize(&self) -> Vec<u8> {
        "_\r\n".as_bytes().to_vec()
    }
}

impl Serialize for Boolean {
    fn serialize(&self) -> Vec<u8> {
        if self.value {
            "#t\r\n".as_bytes().to_vec()
        } else {
            "#f\r\n".as_bytes().to_vec()
        }
    }
}

impl Serialize for Double {
    fn serialize(&self) -> Vec<u8> {
        if self.value.abs() < 1e8 && self.value.abs() > 1e-5 {
            format!(",{}\r\n", self.value).as_bytes().to_vec()
        } else {
            format!(",{:+e}\r\n", self.value).as_bytes().to_vec()
        }
    }
}

impl Serialize for BulkError {
    fn serialize(&self) -> Vec<u8> {
        format!("!{}\r\n{}\r\n", self.value.len(), self.value)
            .as_bytes()
            .to_vec()
    }
}

impl Serialize for Array {
    fn serialize(&self) -> Vec<u8> {
        let mut result = format!("*{}\r\n", self.value.len()).as_bytes().to_vec();
        for item in &self.value {
            result.extend(item.serialize());
        }
        result
    }
}

impl Serialize for Map {
    fn serialize(&self) -> Vec<u8> {
        let mut result = format!("%{}\r\n", self.len()).as_bytes().to_vec();
        for (k, v) in self.iter() {
            result.extend(k.serialize());
            result.extend(v.serialize());
        }
        result
    }
}

impl Serialize for Set {
    fn serialize(&self) -> Vec<u8> {
        let mut result = format!("~{}\r\n", self.len()).as_bytes().to_vec();
        for k in self.iter() {
            result.extend(k.serialize());
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_simple_string() {
        let s = SimpleString {
            value: "OK".to_string(),
        };
        assert_eq!(s.serialize(), "+OK\r\n".as_bytes());
    }

    #[test]
    fn test_serialize_simple_error() {
        let s = SimpleError {
            value: "ERR".to_string(),
        };
        assert_eq!(s.serialize(), "-ERR\r\n".as_bytes());
    }

    #[test]
    fn test_serialze_integer() {
        let s = Integer { value: 123 };
        assert_eq!(s.serialize(), ":123\r\n".as_bytes());
    }

    #[test]
    fn test_serialize_negative_integer() {
        let s = Integer { value: -123 };
        assert_eq!(s.serialize(), ":-123\r\n".as_bytes());
    }

    #[test]
    fn test_serialize_bulk_string() {
        let s = BulkString {
            value: "foobar".to_string(),
        };
        assert_eq!(s.serialize(), "$6\r\nfoobar\r\n".as_bytes());

        let s = BulkString {
            value: "".to_string(),
        };
        assert_eq!(s.serialize(), "$0\r\n\r\n".as_bytes());
    }

    #[test]
    fn test_serialize_null() {
        let s = Null {};
        assert_eq!(s.serialize(), "_\r\n".as_bytes());
    }

    #[test]
    fn test_serialize_boolean() {
        let s = Boolean { value: true };
        assert_eq!(s.serialize(), "#t\r\n".as_bytes());

        let s = Boolean { value: false };
        assert_eq!(s.serialize(), "#f\r\n".as_bytes());
    }

    #[test]
    fn test_serialize_double() {
        let s = Double { value: 3.99 };
        assert_eq!(s.serialize(), ",3.99\r\n".as_bytes());

        let s = Double { value: -3.88 };
        assert_eq!(s.serialize(), ",-3.88\r\n".as_bytes());

        let s = Double { value: 123400000.0 };
        assert_eq!(s.serialize(), ",+1.234e8\r\n".as_bytes());

        let s = Double { value: -0.00000074 };
        assert_eq!(s.serialize(), ",-7.4e-7\r\n".as_bytes());
    }

    #[test]
    fn test_serialize_bulk_error() {
        let s = BulkError {
            value: "SYNTAX invalid syntax".to_string(),
        };
        assert_eq!(s.serialize(), "!21\r\nSYNTAX invalid syntax\r\n".as_bytes());
    }

    #[test]
    fn test_serialize_array() {
        let s = Array {
            value: vec![
                Resp::SimpleString(SimpleString::new("OK")),
                Resp::Integer(Integer::new(123)),
                Resp::BulkString(BulkString::new("foobar")),
            ],
        };
        assert_eq!(
            s.serialize(),
            "*3\r\n+OK\r\n:123\r\n$6\r\nfoobar\r\n".as_bytes()
        );
    }

    #[test]
    fn test_serialize_map() {
        let mut s = Map::default();
        s.insert(
            Key::SimpleString(SimpleString::new("value1")),
            Resp::BulkError(BulkError::new("error")),
        );
        s.insert(
            Key::Boolean(Boolean::new(true)),
            Resp::Array(Array::default()),
        );
        assert_eq!(
            s.serialize(),
            "%2\r\n+value1\r\n!5\r\nerror\r\n#t\r\n*0\r\n".as_bytes()
        );
    }

    #[test]
    fn test_serialize_set() {
        let mut s = Set::default();
        assert_eq!(s.serialize(), "~0\r\n".as_bytes());

        s.insert(Key::SimpleString(SimpleString::new("value1")));
        s.insert(Key::Boolean(Boolean::default()));

        assert_eq!(s.serialize(), "~2\r\n+value1\r\n#f\r\n".as_bytes());
    }
}
