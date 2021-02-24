use json_decode::{BoxDecoder, DecodeError, Decoder};
use std::marker::PhantomData;

use crate::{codable::Codable, SerializableArgument, SerializeError};

/*
pub trait Scalar: Sized {
>>>>>>> f5ffc35 (First pass - got the scalar type defined, updated it's generation etc.)
    fn decode(value: &serde_json::Value) -> Result<Self, DecodeError>;
    fn encode(&self) -> Result<serde_json::Value, SerializeError>;
}*/

// Something like this?
pub trait Scalar<TypeLock>: Sized + SerializableArgument {
    type Codable: Codable;

    fn from_codable(x: Self::Codable) -> Result<Self, DecodeError>;
    fn to_codable(&self) -> Result<&Self::Codable, SerializeError>;
}

/*
impl<S, TypeLock> Scalar<Option<TypeLock>> for Option<S>
where
    S: Scalar<TypeLock>,
{
    type Codable = Option<S>;

    fn from_codable(x: Self::Codable) -> Result<Self, DecodeError> {
        todo!()
    }

    fn to_codable(&self) -> Result<&Self::Codable, SerializeError> {
        todo!()
    }
}
*/

pub fn decoder<'a, S, TypeLock>() -> BoxDecoder<'a, S>
where
    S: Scalar<TypeLock> + 'a + Send + Sync,
    TypeLock: 'a + Send + Sync,
{
    Box::new(ScalarDecoder {
        phantom: PhantomData,
    })
}

macro_rules! impl_scalar_for {
    ($type:ty) => {
        impl Scalar<$type> for $type {
            type Codable = $type;

            fn from_codable(x: $type) -> Result<$type, DecodeError> {
                Ok(x)
            }

            fn to_codable(&self) -> Result<&$type, SerializeError> {
                Ok(&self)
            }
        }
    };
}

impl_scalar_for!(i32);
impl_scalar_for!(f64);
impl_scalar_for!(bool);
impl_scalar_for!(String);

impl_scalar_for!(serde_json::Value);
crate::impl_serializable_argument_for_scalar!(serde_json::Value);

struct ScalarDecoder<S, T> {
    phantom: PhantomData<(S, T)>,
}

impl<'a, S, TypeLock> Decoder<'a, S> for ScalarDecoder<S, TypeLock>
where
    S: Scalar<TypeLock> + Sized,
{
    fn decode(&self, value: &serde_json::Value) -> Result<S, DecodeError> {
        S::from_codable(S::Codable::decode(value)?)
    }
}
