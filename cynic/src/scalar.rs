use json_decode::{BoxDecoder, DecodeError, Decoder};
use std::marker::PhantomData;

use crate::SerializeError;

pub trait Scalar: Sized {
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError>;
    fn encode(&self) -> Result<serde_json::Value, SerializeError>;
}

pub fn decoder<'a, S>() -> BoxDecoder<'a, S>
where
    S: Scalar + 'a + Send + Sync,
{
    Box::new(ScalarDecoder {
        phantom: PhantomData,
    })
}

impl Scalar for i64 {
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError> {
        json_decode::integer().decode(value)
    }

    fn encode(&self) -> Result<serde_json::Value, SerializeError> {
        Ok((*self).into())
    }
}

impl Scalar for f64 {
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError> {
        json_decode::float().decode(value)
    }

    fn encode(&self) -> Result<serde_json::Value, SerializeError> {
        Ok((*self).into())
    }
}

impl Scalar for bool {
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError> {
        json_decode::boolean().decode(value)
    }

    fn encode(&self) -> Result<serde_json::Value, SerializeError> {
        Ok((*self).into())
    }
}

impl Scalar for String {
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError> {
        json_decode::string().decode(value)
    }

    fn encode(&self) -> Result<serde_json::Value, SerializeError> {
        Ok(self.clone().into())
    }
}

impl Scalar for serde_json::Value {
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError> {
        json_decode::json().decode(value)
    }

    fn encode(&self) -> Result<serde_json::Value, SerializeError> {
        Ok(self.clone())
    }
}

struct ScalarDecoder<S: Scalar> {
    phantom: PhantomData<S>,
}

impl<'a, S> Decoder<'a, S> for ScalarDecoder<S>
where
    S: Scalar + Sized,
{
    fn decode(&self, value: &serde_json::Value) -> Result<S, DecodeError> {
        S::decode(value)
    }
}

#[cfg(feature = "chrono")]
impl Scalar for chrono::DateTime<chrono::FixedOffset> {
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError> {
        use chrono::DateTime;

        match value {
            serde_json::Value::String(s) => {
                Ok(DateTime::parse_from_rfc3339(s).map_err(chrono_decode_error)?)
            }
            _ => Err(DecodeError::IncorrectType(
                "String".to_string(),
                value.to_string(),
            )),
        }
    }

    fn encode(&self) -> Result<serde_json::Value, SerializeError> {
        Ok(serde_json::Value::String(self.to_rfc3339()))
    }
}

#[cfg(feature = "chrono")]
impl Scalar for chrono::DateTime<chrono::Utc> {
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError> {
        use chrono::{DateTime, Utc};

        match value {
            serde_json::Value::String(s) => Ok(DateTime::parse_from_rfc3339(s)
                .map_err(chrono_decode_error)?
                .with_timezone(&Utc)),
            _ => Err(DecodeError::IncorrectType(
                "String".to_string(),
                value.to_string(),
            )),
        }
    }

    fn encode(&self) -> Result<serde_json::Value, SerializeError> {
        Ok(serde_json::Value::String(self.to_rfc3339()))
    }
}

#[cfg(feature = "chrono")]
fn chrono_decode_error(err: chrono::ParseError) -> DecodeError {
    DecodeError::Other(err.to_string())
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "chrono")]
    #[test]
    fn test_utc_datetime_scalar() {
        use super::*;
        use chrono::{DateTime, Utc};

        let datetime: DateTime<Utc> = Utc::now();

        assert_eq!(DateTime::decode(&datetime.encode().unwrap()), Ok(datetime));
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn test_fixed_offset_datetime_scalar() {
        use super::*;
        use chrono::{DateTime, FixedOffset, TimeZone};

        let datetime: DateTime<FixedOffset> = FixedOffset::east(3600 * 5)
            .ymd(2016, 11, 08)
            .and_hms(10, 15, 20);

        assert_eq!(DateTime::decode(&datetime.encode().unwrap()), Ok(datetime));
    }
}
