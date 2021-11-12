use json_decode::{BoxDecoder, DecodeError, Decoder};
use std::marker::PhantomData;

/// Indicates that a type can be used as a scalar of `TypeLock` in a graphql query
pub trait Scalar<TypeLock>: Sized + serde::Serialize {
    /// The type that this scalar should initially be deserialized as.
    ///
    /// Note that this doesn't have to be the actual type of the `Scalar`, though it can be.
    /// Further conversion can be done in `from_deserialize`
    type Deserialize: serde::de::DeserializeOwned;

    /// Converts `Deserialize` into `Self` (this can just be a no-op if `Deserialize` is `Self`)
    fn from_deserialize(x: Self::Deserialize) -> Result<Self, DecodeError>;
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

/// Implements [`cynic::Scalar`] for a given type & type lock.
///
/// For example, to use `uuid::Uuid` for a `Uuid` type defined in a schema:
///
/// ```rust
/// # #[macro_use] extern crate cynic;
/// # // Faking the schema & chrono module here because it's easier than
/// # // actually defining them
/// #
/// # mod schema { pub struct Uuid {} }
/// # mod uuid { pub type Uuid = String; }
/// impl_scalar!(uuid::Uuid, schema::Uuid);
/// ```
///
/// This macro can be used on any type that implements `serde::Serialize`,
/// provided the `schema` is defined in the current crate
#[macro_export]
macro_rules! impl_scalar {
    ($type:path, $type_lock:path) => {
        impl $crate::Scalar<$type_lock> for $type {
            type Deserialize = $type;

            fn from_deserialize(x: $type) -> Result<$type, $crate::DecodeError> {
                Ok(x)
            }
        }

        $crate::impl_input_type!($type, $type_lock);
    };
}

impl_scalar!(i32, i32);
impl_scalar!(f64, f64);
impl_scalar!(bool, bool);
impl_scalar!(String, String);

impl_scalar!(serde_json::Value, serde_json::Value);

struct ScalarDecoder<S, T> {
    phantom: PhantomData<(S, T)>,
}

impl<'a, S, TypeLock> Decoder<'a, S> for ScalarDecoder<S, TypeLock>
where
    S: Scalar<TypeLock> + Sized,
{
    fn decode(&self, value: &serde_json::Value) -> Result<S, DecodeError> {
        S::from_deserialize(
            serde_json::from_value(value.clone())
                .map_err(|e| DecodeError::SerdeError(e.to_string()))?,
        )
    }
}
