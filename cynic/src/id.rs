use crate::SerializeError;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Id(String);

impl Id {
    pub fn new(s: impl Into<String>) -> Self {
        Id(s.into())
    }
    pub fn inner(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl<T: Into<String>> From<T> for Id {
    fn from(s: T) -> Id {
        Id(s.into())
    }
}

impl crate::Scalar for Id {
    fn decode(value: &serde_json::Value) -> Result<Self, json_decode::DecodeError> {
        String::decode(value).map(Into::into)
    }

    fn encode(&self) -> Result<serde_json::Value, SerializeError> {
        self.0.encode()
    }
}
