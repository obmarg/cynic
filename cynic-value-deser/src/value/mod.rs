mod enums;
mod lists;
mod objects;
mod scalars;
mod value;

pub use {
    enums::EnumValue,
    lists::{List, ListIter},
    objects::{FieldIter, Object, ObjectField},
    scalars::{BooleanValue, FloatValue, IntValue, NullValue, StringValue},
    value::DeserValue,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ValueType {
    Int,
    Float,
    String,
    Boolean,
    Null,
    Enum,
    List,
    Object,
}

impl From<DeserValue<'_>> for ValueType {
    fn from(value: DeserValue<'_>) -> Self {
        match value {
            DeserValue::Int(_) => ValueType::Int,
            DeserValue::Float(_) => ValueType::Float,
            DeserValue::String(_) => ValueType::String,
            DeserValue::Boolean(_) => ValueType::Boolean,
            DeserValue::Null(_) => ValueType::Null,
            DeserValue::Enum(_) => ValueType::Enum,
            DeserValue::List(_) => ValueType::List,
            DeserValue::Object(_) => ValueType::Object,
        }
    }
}
