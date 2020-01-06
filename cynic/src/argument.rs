pub struct Argument {
    pub(crate) name: String,
    pub(crate) value: serde_json::Value,
}

impl Argument {
    pub fn new(name: &str, value: serde_json::Value) -> Self {
        Argument {
            name: name.to_string(),
            value,
        }
    }
}
