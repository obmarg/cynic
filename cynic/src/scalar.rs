use json_decode::{BoxDecoder, DecodeError, Decoder};
use std::marker::PhantomData;

use crate::{SerializableArgument, SerializeError};

pub trait Scalar<TypeLock>: Sized + SerializableArgument {
    type Serialize: serde::Serialize + serde::de::DeserializeOwned;

    fn from_serialize(x: Self::Serialize) -> Result<Self, DecodeError>;
    fn to_serialize(&self) -> Result<Self::Serialize, SerializeError>;
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

// TODO: Docs
#[macro_export]
macro_rules! impl_scalar {
    ($type_lock:path, $type:path) => {
        impl $crate::Scalar<$type_lock> for $type {
            type Serialize = $type;

            fn from_serialize(x: $type) -> Result<$type, $crate::DecodeError> {
                Ok(x)
            }

            fn to_serialize(&self) -> Result<$type, $crate::SerializeError> {
                Ok(self.clone())
            }
        }
    };
}

impl_scalar!(i32, i32);
impl_scalar!(f64, f64);
impl_scalar!(bool, bool);
impl_scalar!(String, String);

impl_scalar!(serde_json::Value, serde_json::Value);
crate::impl_serializable_argument_for_scalar!(serde_json::Value);

struct ScalarDecoder<S, T> {
    phantom: PhantomData<(S, T)>,
}

impl<'a, S, TypeLock> Decoder<'a, S> for ScalarDecoder<S, TypeLock>
where
    S: Scalar<TypeLock> + Sized,
{
    fn decode(&self, value: &serde_json::Value) -> Result<S, DecodeError> {
        S::from_serialize(
            serde_json::from_value(value.clone())
                .map_err(|e| DecodeError::SerdeError(e.to_string()))?,
        )
    }
}
