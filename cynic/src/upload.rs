use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Upload {
    pub(crate) name: String,
    pub(crate) content: Vec<u8>,
}

impl Upload {
    pub fn new(name: String, content: Vec<u8>) -> Self {
        Self { name, content }
    }
}

// TODO: Required?
impl crate::Scalar<Upload> for Upload {
    type Deserialize = String;

    fn from_deserialize(_s: String) -> Result<Self, json_decode::DecodeError> {
        Ok(Upload {
            name: String::new(),
            content: vec![],
        })
    }
}
