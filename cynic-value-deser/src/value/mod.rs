mod enums;
mod lists;
mod objects;
mod scalars;
mod value;

use std::fmt::Display;

use cynic_parser::ConstValue;
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

impl Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
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

impl From<ConstValue<'_>> for ValueType {
    fn from(value: ConstValue<'_>) -> Self {
        match value {
            ConstValue::Int(_) => ValueType::Int,
            ConstValue::Float(_) => ValueType::Float,
            ConstValue::String(_) => ValueType::String,
            ConstValue::Boolean(_) => ValueType::Boolean,
            ConstValue::Null(_) => ValueType::Null,
            ConstValue::Enum(_) => ValueType::Enum,
            ConstValue::List(_) => ValueType::List,
            ConstValue::Object(_) => ValueType::Object,
        }
    }
}
