use json_decode::DecodeError;

use crate::SerializeError;

/// A raw value that can be decoded & encoded.
///
/// Users shouldn't usually need to implement this themselves - implementations
/// are provided for all the normal JSON types.  If users need a custom scalar
/// they should usually use the `Scalar` trait to implement it using an existing
/// `Codable` instance.
pub trait Codable: Sized {
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError>;
    fn encode(&self) -> Result<serde_json::Value, SerializeError>;
}

impl Codable for i32 {
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError> {
        json_decode::integer().decode(value)
    }

    fn encode(&self) -> Result<serde_json::Value, SerializeError> {
        Ok((*self).into())
    }
}

impl Codable for f64 {
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError> {
        json_decode::float().decode(value)
    }

    fn encode(&self) -> Result<serde_json::Value, SerializeError> {
        Ok((*self).into())
    }
}

impl Codable for bool {
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError> {
        json_decode::boolean().decode(value)
    }

    fn encode(&self) -> Result<serde_json::Value, SerializeError> {
        Ok((*self).into())
    }
}

impl Codable for String {
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError> {
        json_decode::string().decode(value)
    }

    fn encode(&self) -> Result<serde_json::Value, SerializeError> {
        Ok(self.clone().into())
    }
}

impl Codable for serde_json::Value {
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError> {
        json_decode::json().decode(value)
    }

    fn encode(&self) -> Result<serde_json::Value, SerializeError> {
        Ok(self.clone())
    }
}

impl<T: Codable> Codable for Vec<T> {
    fn decode(_value: &serde_json::Value) -> Result<Self, DecodeError> {
        todo!()
    }

    fn encode(&self) -> Result<serde_json::Value, SerializeError> {
        self.iter()
            .map(|s| s.encode())
            .collect::<Result<Vec<_>, _>>()
            .map(serde_json::Value::Array)
    }
}

impl<T: Codable> Codable for Option<T> {
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError> {
        match value {
            serde_json::Value::Null => Ok(None),
            _ => Ok(Some(T::decode(value)?)),
        }
    }

    fn encode(&self) -> Result<serde_json::Value, SerializeError> {
        match self {
            Some(inner) => Ok(inner.encode()?),
            None => Ok(serde_json::Value::Null),
        }
    }
}

impl<T: Codable> Codable for Box<T> {
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError> {
        Ok(Box::new(T::decode(value)?))
    }

    fn encode(&self) -> Result<serde_json::Value, SerializeError> {
        self.as_ref().encode()
    }
}
