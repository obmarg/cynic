use serde::{Deserialize, Serialize};

/// A GraphQL `ID`
///
/// Any field in a GraphQL schema that has the type `ID` should be represented by
/// this struct.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Id(String);

impl Id {
    /// Constructs an `ID` from a `String`, `&str` or similar
    ///
    /// ```
    /// cynic::Id::new("123");
    /// ```
    pub fn new(s: impl Into<String>) -> Self {
        Id(s.into())
    }

    /// Returns a reference to the value of this `Id`
    pub fn inner(&self) -> &str {
        &self.0
    }

    /// Converts this `Id` into its inner value
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl<T: Into<String>> From<T> for Id {
    fn from(s: T) -> Id {
        Id(s.into())
    }
}
