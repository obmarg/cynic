use bson::oid::ObjectId;
use json_decode::DecodeError;

use crate::{scalar::Scalar, SerializeError};

impl Scalar for ObjectId {
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError> {
        match value {
            serde_json::Value::String(s) => {
                Ok(ObjectId::with_string(s).map_err(|err| DecodeError::Other(err.to_string()))?)
            }
            _ => Err(DecodeError::IncorrectType(
                "String".to_string(),
                value.to_string(),
            )),
        }
    }

    fn encode(&self) -> Result<serde_json::Value, SerializeError> {
        Ok(serde_json::Value::String(self.to_hex()))
    }
}

crate::impl_serializable_argument_for_scalar!(ObjectId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bson_object_id_scalar() {
        let id: ObjectId = ObjectId::new();

        assert_eq!(ObjectId::decode(&id.encode().unwrap()), Ok(id));
    }
}
