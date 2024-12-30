mod deserialize;
mod deserializers;
mod error;

pub mod value;

pub use cynic_parser::Span;

pub use {
    self::value::DeserValue,
    deserialize::{ValueDeserialize, ValueDeserializeOwned},
    deserializers::ConstDeserializer,
    error::Error,
};

pub use cynic_parser_deser_macros::ValueDeserialize;
