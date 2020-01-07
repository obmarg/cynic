use json_decode::{DecodeError, Decoder};
use std::marker::PhantomData;

pub trait Scalar: Sized {
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError>;
}

pub fn decoder<'a, S>() -> Box<dyn Decoder<'a, S> + 'a>
where
    S: Scalar + 'a,
{
    Box::new(ScalarDecoder {
        phantom: PhantomData,
    })
}

impl Scalar for i64 {
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError> {
        json_decode::integer().decode(value)
    }
}

impl Scalar for f64 {
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError> {
        json_decode::float().decode(value)
    }
}

impl Scalar for bool {
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError> {
        json_decode::boolean().decode(value)
    }
}

impl Scalar for String {
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError> {
        json_decode::string().decode(value)
    }
}

impl Scalar for serde_json::Value {
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError> {
        json_decode::json().decode(value)
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
