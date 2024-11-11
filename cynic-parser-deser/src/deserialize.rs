use std::collections::HashMap;

use crate::{value::ValueType, ConstDeserializer, DeserValue, Error};

// ValueDeserialize vs DeserializeValue
pub trait ValueDeserialize<'a>: Sized {
    fn deserialize(input: DeserValue<'a>) -> Result<Self, Error>;
}

pub trait ValueDeserializeOwned: for<'a> ValueDeserialize<'a> {}
impl<T> ValueDeserializeOwned for T where T: for<'a> ValueDeserialize<'a> {}

impl<'a> ValueDeserialize<'a> for String {
    fn deserialize(input: DeserValue<'a>) -> Result<Self, Error> {
        match input {
            DeserValue::String(string_value) => Ok(string_value.as_str().to_string()),
            other => Err(Error::unexpected_type(ValueType::String, other)),
        }
    }
}

impl<'a> ValueDeserialize<'a> for &'a str {
    fn deserialize(input: DeserValue<'a>) -> Result<Self, Error> {
        match input {
            DeserValue::String(string_value) => Ok(string_value.as_str()),
            other => Err(Error::unexpected_type(ValueType::String, other)),
        }
    }
}

impl<'a> ValueDeserialize<'a> for i32 {
    fn deserialize(input: DeserValue<'a>) -> Result<Self, Error> {
        match input {
            DeserValue::Int(inner) => Ok(inner.as_i32()),
            other => Err(Error::unexpected_type(ValueType::Int, other)),
        }
    }
}

impl<'a> ValueDeserialize<'a> for i64 {
    fn deserialize(input: DeserValue<'a>) -> Result<Self, Error> {
        match input {
            DeserValue::Int(inner) => Ok(inner.as_i64()),
            other => Err(Error::unexpected_type(ValueType::Int, other)),
        }
    }
}

impl<'a> ValueDeserialize<'a> for u32 {
    fn deserialize(input: DeserValue<'a>) -> Result<Self, Error> {
        let value = i64::deserialize(input)?;

        if value < 0 {
            return Err(Error::custom(
                format!("integer was less than zero: {value}"),
                input.span(),
            ));
        }

        value
            .try_into()
            .map_err(|_| Error::custom(format!("integer was too large: {value}"), input.span()))
    }
}

impl<'a> ValueDeserialize<'a> for u64 {
    fn deserialize(input: DeserValue<'a>) -> Result<Self, Error> {
        let value = i64::deserialize(input)?;

        if value < 0 {
            return Err(Error::custom(
                format!("integer was less than zero: {value}"),
                input.span(),
            ));
        }

        value
            .try_into()
            .map_err(|_| Error::custom(format!("integer was too large: {value}"), input.span()))
    }
}

impl<'a> ValueDeserialize<'a> for usize {
    fn deserialize(input: DeserValue<'a>) -> Result<Self, Error> {
        let value = i64::deserialize(input)?;

        if value < 0 {
            return Err(Error::custom(
                format!("integer was less than zero: {value}"),
                input.span(),
            ));
        }

        value
            .try_into()
            .map_err(|_| Error::custom(format!("integer was too large: {value}"), input.span()))
    }
}

impl<'a> ValueDeserialize<'a> for f64 {
    fn deserialize(input: DeserValue<'a>) -> Result<Self, Error> {
        match input {
            DeserValue::Float(inner) => Ok(inner.as_f64()),
            other => Err(Error::unexpected_type(ValueType::Float, other)),
        }
    }
}

impl<'a> ValueDeserialize<'a> for bool {
    fn deserialize(input: DeserValue<'a>) -> Result<Self, Error> {
        match input {
            DeserValue::Boolean(inner) => Ok(inner.as_bool()),
            other => Err(Error::unexpected_type(ValueType::Boolean, other)),
        }
    }
}

impl<'a> ValueDeserialize<'a> for () {
    fn deserialize(input: DeserValue<'a>) -> Result<Self, Error> {
        match input {
            DeserValue::Null(_) => Ok(()),
            other => Err(Error::unexpected_type(ValueType::Null, other)),
        }
    }
}

impl<'a, T> ValueDeserialize<'a> for Option<T>
where
    T: ValueDeserialize<'a>,
{
    fn deserialize(input: DeserValue<'a>) -> Result<Self, Error> {
        match input {
            DeserValue::Null(_) => Ok(None),
            other => T::deserialize(other).map(Some),
        }
    }
}

impl<'a, T> ValueDeserialize<'a> for Vec<T>
where
    T: ValueDeserialize<'a>,
{
    fn deserialize(input: DeserValue<'a>) -> Result<Self, Error> {
        match input {
            DeserValue::List(list) => list.items().map(T::deserialize).collect(),
            other => Err(Error::unexpected_type(ValueType::List, other)),
        }
    }
}

impl<'a, T> ValueDeserialize<'a> for HashMap<String, T>
where
    T: ValueDeserialize<'a>,
{
    fn deserialize(input: DeserValue<'a>) -> Result<Self, Error> {
        match input {
            DeserValue::Object(object) => Ok(object
                .fields()
                .map(|field| Ok((field.name().to_string(), field.value().deserialize()?)))
                .collect::<Result<HashMap<_, _>, _>>()?),
            other => Err(Error::unexpected_type(ValueType::Object, other)),
        }
    }
}

impl<'a, T> ValueDeserialize<'a> for HashMap<&'a str, T>
where
    T: ValueDeserialize<'a>,
{
    fn deserialize(input: DeserValue<'a>) -> Result<Self, Error> {
        match input {
            DeserValue::Object(object) => Ok(object
                .fields()
                .map(|field| Ok((field.name(), field.value().deserialize()?)))
                .collect::<Result<HashMap<_, _>, _>>()?),
            other => Err(Error::unexpected_type(ValueType::Object, other)),
        }
    }
}
