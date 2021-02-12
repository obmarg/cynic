use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime, Utc};
use json_decode::DecodeError;

use crate::{scalar::Scalar, SerializeError};

impl Scalar for NaiveDate {
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError> {
        match value {
            serde_json::Value::String(s) => Ok(NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map_err(|e| chrono_parse_error(e, "NaiveDate"))?),
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

crate::impl_serializable_argument_for_scalar!(NaiveDate);

impl Scalar for NaiveTime {
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError> {
        match value {
            serde_json::Value::String(s) => {
                match s.parse() {
                    Ok(s) => Ok(s),
                    Err(e) => {
                        // Attempt to fall back to times without a second component.
                        match NaiveTime::parse_from_str(s, "%H:%M") {
                            Ok(s) => Ok(s),
                            _ => Err(chrono_parse_error(e, "NaiveTime")),
                        }
                    }
                }
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

crate::impl_serializable_argument_for_scalar!(NaiveTime);

impl Scalar for DateTime<FixedOffset> {
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError> {
        match value {
            serde_json::Value::String(s) => Ok(DateTime::parse_from_rfc3339(s)
                .map_err(|e| chrono_parse_error(e, "DateTime<FixedOffset>"))?),
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
                .map_err(|e| chrono_parse_error(e, "DateTime<Utc>"))?
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

fn chrono_parse_error(err: chrono::ParseError, kind: &'static str) -> DecodeError {
    DecodeError::Other(format!("{} parse error: {}", kind, err.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_naive_date_scalar() {
        use chrono::NaiveDate;

        let date: NaiveDate = NaiveDate::from_ymd(2020, 12, 29);

        assert_eq!(NaiveDate::decode(&date.encode().unwrap()), Ok(date));
    }

    #[test]
    fn test_naive_time_scalar() {
        use chrono::NaiveTime;

        let time: NaiveTime = NaiveTime::from_hms(15, 3, 19);
        assert_eq!(NaiveTime::decode(&time.encode().unwrap()), Ok(time));

        let time_with_millis: NaiveTime = NaiveTime::from_hms_milli(15, 3, 10, 234);
        assert_eq!(
            NaiveTime::decode(&time_with_millis.encode().unwrap()),
            Ok(time_with_millis)
        );
    }

    #[test]
    fn test_naive_time_decode_supports_short_forms() {
        assert_eq!(
            NaiveTime::decode(&json!("10:00:43.123")),
            Ok(NaiveTime::from_hms_milli(10, 0, 43, 123))
        );
        assert_eq!(
            NaiveTime::decode(&json!("10:00:43")),
            Ok(NaiveTime::from_hms(10, 0, 43))
        );
        assert_eq!(
            NaiveTime::decode(&json!("10:05")),
            Ok(NaiveTime::from_hms(10, 5, 0))
        );
    }

    #[test]
    fn test_fixed_offset_datetime_scalar() {
        use chrono::TimeZone;

        let datetime: DateTime<FixedOffset> = FixedOffset::east(3600 * 5)
            .ymd(2016, 11, 8)
            .and_hms(10, 15, 20);

        assert_eq!(DateTime::decode(&datetime.encode().unwrap()), Ok(datetime));
    }

    #[test]
    fn test_utc_datetime_scalar() {
        let datetime: DateTime<Utc> = Utc::now();

        assert_eq!(DateTime::decode(&datetime.encode().unwrap()), Ok(datetime));
    }
}
