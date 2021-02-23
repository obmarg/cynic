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

impl crate::Scalar<Id> for Id {
    type Codable = String;

    fn from_codable(s: String) -> Result<Self, json_decode::DecodeError> {
        Ok(s.into())
    }

    fn to_codable(&self) -> Result<&Self::Codable, SerializeError> {
        Ok(&self.0)
    }
}
