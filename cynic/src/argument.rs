use crate::{Enum, InputObject, Scalar};

pub struct Argument {
    pub(crate) name: String,
    pub(crate) value: Result<serde_json::Value, ()>,
    pub(crate) type_: String,
}

impl Argument {
    pub fn new(name: &str, gql_type: &str, value: serde_json::Value) -> Self {
        Argument {
            name: name.to_string(),
            value: Ok(value),
            type_: gql_type.to_string(),
        }
    }

    pub fn from_serializable(
        name: &str,
        gql_type: &str,
        value: impl SerializableArgument,
    ) -> Argument {
        Argument {
            name: name.to_string(),
            value: value.serialize(),
            type_: gql_type.to_string(),
        }
    }
}

pub trait SerializableArgument {
    fn serialize(&self) -> Result<serde_json::Value, ()>;
}

// All Input objects are serializable.
impl<TypeLock> SerializableArgument for dyn crate::InputObject<TypeLock> {
    fn serialize(&self) -> Result<serde_json::Value, ()> {
        self.serialize()
    }
}

impl<T: SerializableArgument> SerializableArgument for Vec<T> {
    fn serialize(&self) -> Result<serde_json::Value, ()> {
        self.iter()
            .map(|s| s.serialize())
            .collect::<Result<Vec<_>, _>>()
            .map(serde_json::Value::Array)
    }
}

impl<T: SerializableArgument> SerializableArgument for Option<T> {
    fn serialize(&self) -> Result<serde_json::Value, ()> {
        match self {
            Some(inner) => Ok(inner.serialize()?),
            None => Ok(serde_json::Value::Null),
        }
    }
}

impl<T: Scalar> SerializableArgument for T {
    fn serialize(&self) -> Result<serde_json::Value, ()> {
        self.encode()
    }
}
