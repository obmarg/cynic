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

    pub fn new_serialize<V: serde::Serialize>(name: &str, gql_type: &str, value: V) -> Self {
        Argument {
            name: name.to_string(),
            // TODO: should actually pass up the Err here...
            value: serde_json::to_value(value).map_err(|_| ()),
            type_: gql_type.to_string(),
        }
    }
}
