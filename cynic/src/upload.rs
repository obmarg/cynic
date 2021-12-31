use serde::{Deserialize, Serialize};

/// A file upload handle to transmit files via GraphQL/HTTP Multipart requests.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Upload {
    pub(crate) name: String,
    pub(crate) content: Vec<u8>,
}

impl Upload {
    /// Creates a new file upload handle.
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
