use crate::SerializeError;

pub struct Argument {
    pub(crate) name: String,
    pub(crate) serialize_result: Result<serde_json::Value, SerializeError>,
    pub(crate) type_: String,
}

impl Argument {
    pub fn new(name: &str, gql_type: &str, value: impl SerializableArgument) -> Argument {
        Argument {
            name: name.to_string(),
            serialize_result: value.serialize(),
            type_: gql_type.to_string(),
        }
    }
}

impl serde::Serialize for Argument {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::Error;

        match &self.serialize_result {
            Ok(json_val) => serde::Serialize::serialize(json_val, serializer),
            Err(e) => Err(S::Error::custom(e.to_string())),
        }
    }
}

pub trait SerializableArgument {
    fn serialize(&self) -> Result<serde_json::Value, SerializeError>;
}

impl<T: SerializableArgument> SerializableArgument for Vec<T> {
    fn serialize(&self) -> Result<serde_json::Value, SerializeError> {
        self.iter()
            .map(|s| s.serialize())
            .collect::<Result<Vec<_>, _>>()
            .map(serde_json::Value::Array)
    }
}

impl<T: SerializableArgument> SerializableArgument for Option<T> {
    fn serialize(&self) -> Result<serde_json::Value, SerializeError> {
        match self {
            Some(inner) => Ok(inner.serialize()?),
            None => Ok(serde_json::Value::Null),
        }
    }
}

impl<T: SerializableArgument> SerializableArgument for Box<T> {
    fn serialize(&self) -> Result<serde_json::Value, SerializeError> {
        self.as_ref().serialize()
    }
}

impl<T: SerializableArgument> SerializableArgument for &T {
    fn serialize(&self) -> Result<serde_json::Value, SerializeError> {
        (*self).serialize()
    }
}

impl SerializableArgument for &str {
    fn serialize(&self) -> Result<serde_json::Value, SerializeError> {
        Ok(serde_json::Value::String(self.to_string()))
    }
}

// TODO: Consider how much of this stuff is actually needed now...

/// Implements serializable argument for scalar types.
#[macro_export]
macro_rules! impl_serializable_argument_for_scalar {
    ($inner:ty) => {
        impl $crate::SerializableArgument for $inner {
            fn serialize(&self) -> Result<$crate::serde_json::Value, $crate::SerializeError> {
                use $crate::Scalar;
                Ok($crate::serde_json::to_value(self.to_serialize()?)?)
            }
        }
    };
}

// TODO: Can the above just be generic now?

impl_serializable_argument_for_scalar!(i32);
impl_serializable_argument_for_scalar!(f64);
impl_serializable_argument_for_scalar!(String);
impl_serializable_argument_for_scalar!(bool);
impl_serializable_argument_for_scalar!(crate::Id);
