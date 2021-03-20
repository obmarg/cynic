use json_decode::{BoxDecoder, DecodeError, Decoder};
use std::marker::PhantomData;

use crate::{SerializableArgument, SerializeError};

pub trait Scalar<TypeLock>: Sized + SerializableArgument {
    type Serializable: serde::Serialize + serde::de::DeserializeOwned;

    fn from_serializable(x: Self::Serializable) -> Result<Self, DecodeError>;
    fn to_serializable(&self) -> Result<&Self::Serializable, SerializeError>;
}

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
            type Serializable = $type;

            fn from_serializable(x: $type) -> Result<$type, DecodeError> {
                Ok(x)
            }

            fn to_serializable(&self) -> Result<&$type, SerializeError> {
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
        S::from_serializable(
            serde_json::from_value(value.clone())
                .map_err(|e| DecodeError::SerdeError(e.to_string()))?,
        )
    }
}
