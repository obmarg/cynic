use crate::Scalar;

pub struct Argument {
    pub(crate) name: String,
    pub(crate) value: Box<dyn SerializableArgument>,
    pub(crate) type_: String,
}

impl Argument {
    pub fn new(name: &str, gql_type: &str, value: impl SerializableArgument + 'static) -> Argument {
        Argument {
            name: name.to_string(),
            value: Box::new(value),
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

        match self.value.serialize() {
            Ok(json_val) => serde::Serialize::serialize(&json_val, serializer),
            // TODO: Better error here...
            Err(e) => Err(S::Error::custom("Could not serialize")),
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
