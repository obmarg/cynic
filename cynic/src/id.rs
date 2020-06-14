use crate::SerializeError;

#[derive(Clone, Debug)]
pub struct Id(String);

impl Id {
    pub fn inner(&self) -> &str {
        return &self.0;
    }

    pub fn into_inner(self) -> String {
        return self.0;
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
