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

impl Scalar for i32 {
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
