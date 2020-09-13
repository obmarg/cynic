use json_decode::DecodeError;
use uuid::Uuid;

use crate::{scalar::Scalar, SerializeError};

impl Scalar for Uuid {
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError> {
        match value {
            serde_json::Value::String(s) => {
                Ok(Uuid::parse_str(s).map_err(|err| DecodeError::Other(err.to_string()))?)
            }
            _ => Err(DecodeError::IncorrectType(
                "String".to_string(),
                value.to_string(),
            )),
        }
    }

    fn encode(&self) -> Result<serde_json::Value, SerializeError> {
        Ok(serde_json::Value::String(self.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_scalar() {
        let bytes = [4, 54, 67, 12, 43, 2, 98, 76, 32, 50, 87, 5, 1, 33, 43, 87];
        let id: Uuid = Uuid::from_bytes(bytes);

        assert_eq!(Uuid::decode(&id.encode().unwrap()), Ok(id));
    }
}
