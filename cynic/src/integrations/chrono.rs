use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
use json_decode::DecodeError;

use crate::{scalar::Scalar, SerializeError};

impl Scalar for NaiveDate {
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError> {
        match value {
            serde_json::Value::String(s) => {
                Ok(NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(chrono_decode_error)?)
            }
            _ => Err(DecodeError::IncorrectType(
                "String".to_string(),
                value.to_string(),
            )),
        }
    }

    fn encode(&self) -> Result<serde_json::Value, SerializeError> {
        Ok(serde_json::Value::String(
            self.format("%Y-%m-%d").to_string(),
        ))
    }
}

impl Scalar for DateTime<FixedOffset> {
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError> {
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

crate::impl_serializable_argument_for_scalar!(DateTime<FixedOffset>);

impl Scalar for DateTime<Utc> {
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError> {
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

crate::impl_serializable_argument_for_scalar!(DateTime<Utc>);

fn chrono_decode_error(err: chrono::ParseError) -> DecodeError {
    DecodeError::Other(err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utc_datetime_scalar() {
        let datetime: DateTime<Utc> = Utc::now();

        assert_eq!(DateTime::decode(&datetime.encode().unwrap()), Ok(datetime));
    }

    #[test]
    fn test_fixed_offset_datetime_scalar() {
        use chrono::TimeZone;

        let datetime: DateTime<FixedOffset> = FixedOffset::east(3600 * 5)
            .ymd(2016, 11, 08)
            .and_hms(10, 15, 20);

        assert_eq!(DateTime::decode(&datetime.encode().unwrap()), Ok(datetime));
    }

    #[test]
    fn test_naive_date_scalar() {
        use chrono::NaiveDate;

        let date: NaiveDate = NaiveDate::from_ymd(2020, 12, 29);

        assert_eq!(NaiveDate::decode(&date.encode().unwrap()), Ok(date));
    }
}
