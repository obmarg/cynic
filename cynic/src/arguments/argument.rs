/// A single argument to a graphql operation.
///
/// Users should only need to use this if they're interacting with the
/// `selection_set` API directly.
pub struct Argument {
    pub(crate) name: String,
    pub(crate) serialize_result: Result<serde_json::Value, serde_json::Error>,
    pub(crate) type_: String,
}

impl Argument {
    /// Constructs a new `Argument`
    pub fn new(
        name: &str,
        gql_type: &str,
        result: Result<serde_json::Value, serde_json::Error>,
    ) -> Argument {
        Argument {
            name: name.to_string(),
            serialize_result: result,
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
