use crate::Upload;

#[derive(Debug)]
pub struct Argument {
    pub(crate) name: String,
    pub(crate) wire_format: ArgumentWireFormat,
    pub(crate) type_: String,
}

impl Argument {
    pub fn new(name: &str, gql_type: &str, result: ArgumentWireFormat) -> Argument {
        Argument {
            name: name.to_string(),
            wire_format: result,
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

        let e = match &self.wire_format {
            ArgumentWireFormat::Serialize(serialize) => match serialize {
                Ok(json_val) => serde::Serialize::serialize(json_val, serializer),
                Err(e) => {
                    log::debug!("{:?}", e.to_string());
                    Err(S::Error::custom(e.to_string()))
                }
            },
            ArgumentWireFormat::Upload(_) => Err(S::Error::custom(
                "Upload must not be serialized but sent as multiplart!",
            )),
        };

        e
    }
}

#[derive(Debug)]
pub enum ArgumentWireFormat {
    Serialize(Result<serde_json::Value, serde_json::Error>),
    Upload(Upload),
}
