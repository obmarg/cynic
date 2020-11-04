use json_decode::DecodeError;
use url::Url;

use crate::{scalar::Scalar, SerializeError};

impl Scalar for Url {
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError> {
        match value {
            serde_json::Value::String(s) => {
                Ok(Url::parse(s).map_err(|err| DecodeError::Other(err.to_string()))?)
            }
            _ => Err(DecodeError::IncorrectType(
                "String".to_string(),
                value.to_string(),
            )),
        }
    }

    fn encode(&self) -> Result<serde_json::Value, SerializeError> {
        Ok(serde_json::Value::String(self.as_str().to_owned()))
    }
}

crate::impl_serializable_argument_for_scalar!(Url);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bson_object_id_scalar() {
        let url: Url = Url::parse("https://example.net/").unwrap();

        assert_eq!(Url::decode(&url.encode().unwrap()), Ok(url));
    }
}
