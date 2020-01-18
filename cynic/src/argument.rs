pub struct Argument {
    pub(crate) name: String,
    pub(crate) value: Result<serde_json::Value, ()>,
}

impl Argument {
    pub fn new(name: &str, value: serde_json::Value) -> Self {
        Argument {
            name: name.to_string(),
            value: Ok(value),
        }
    }

    pub fn new_serialize<V: serde::Serialize>(name: &str, value: V) -> Self {
        Argument {
            name: name.to_string(),
            // TODO: should actually pass up the Err here...
            value: serde_json::to_value(value).map_err(|_| ()),
        }
    }
}
