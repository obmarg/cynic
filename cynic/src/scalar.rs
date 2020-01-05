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
