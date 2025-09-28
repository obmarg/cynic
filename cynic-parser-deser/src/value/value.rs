use cynic_parser::{ConstValue, Span};

use super::{
    enums::EnumValue,
    lists::{List, ListIter},
    objects::{FieldIter, Object},
    scalars::{BooleanValue, FloatValue, IntValue, NullValue, StringValue},
};

#[derive(Debug, Copy, Clone)]
pub enum DeserValue<'a> {
    Int(IntValue),
    Float(FloatValue),
    String(StringValue<'a>),
    Boolean(BooleanValue),
    Null(NullValue),
    Enum(EnumValue<'a>),
    List(List<'a>),
    Object(Object<'a>),
}

impl DeserValue<'_> {
    pub fn span(&self) -> Option<Span> {
        match self {
            DeserValue::Int(inner) => inner.span(),
            DeserValue::Float(inner) => inner.span(),
            DeserValue::String(inner) => inner.span(),
            DeserValue::Boolean(inner) => inner.span(),
            DeserValue::Null(inner) => inner.span(),
            DeserValue::Enum(inner) => inner.span(),
            DeserValue::List(inner) => inner.span(),
            DeserValue::Object(inner) => inner.span(),
        }
    }
}

impl<'a> DeserValue<'a> {
    pub fn is_int(&self) -> bool {
        matches!(self, Self::Int(_))
    }

    pub fn as_i32(&self) -> Option<i32> {
        match self {
            Self::Int(inner) => Some(inner.as_i32()),
            _ => None,
        }
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float(_))
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Float(inner) => Some(inner.value()),
            _ => None,
        }
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    pub fn as_str(&self) -> Option<&'a str> {
        match self {
            Self::String(inner) => Some(inner.value()),
            _ => None,
        }
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self, Self::Boolean(_))
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Boolean(boolean_value) => Some(boolean_value.value()),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null(_))
    }

    pub fn as_null(&self) -> Option<()> {
        match self {
            Self::Null(_) => Some(()),
            _ => None,
        }
    }

    pub fn is_enum(&self) -> bool {
        matches!(self, Self::Enum(_))
    }

    pub fn as_enum_value(&self) -> Option<&'a str> {
        match self {
            Self::Enum(value) => Some(value.name()),
            _ => None,
        }
    }

    pub fn is_list(&self) -> bool {
        matches!(self, Self::List(_))
    }

    pub fn as_list(&self) -> Option<List<'a>> {
        match self {
            Self::List(inner) => Some(*inner),
            _ => None,
        }
    }

    pub fn as_items(&self) -> Option<ListIter<'a>> {
        match self {
            Self::List(inner) => Some(inner.items()),
            _ => None,
        }
    }

    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    pub fn as_object(&self) -> Option<Object<'a>> {
        match self {
            Self::Object(inner) => Some(*inner),
            _ => None,
        }
    }

    pub fn as_fields(&self) -> Option<FieldIter<'a>> {
        match self {
            Self::Object(inner) => Some(inner.fields()),
            _ => None,
        }
    }
}

impl<'a> DeserValue<'a> {
    pub fn from_const(value: ConstValue<'a>) -> Self {
        match value {
            ConstValue::Int(val) => DeserValue::Int(IntValue {
                value: val.value(),
                span: Some(val.span()),
            }),
            ConstValue::Float(val) => DeserValue::Float(FloatValue {
                value: val.value(),
                span: Some(val.span()),
            }),
            ConstValue::String(val) => DeserValue::String(StringValue {
                value: val.value(),
                span: Some(val.span()),
            }),
            ConstValue::Boolean(val) => DeserValue::Boolean(BooleanValue {
                value: val.value(),
                span: Some(val.span()),
            }),
            ConstValue::Null(val) => DeserValue::Null(NullValue {
                span: Some(val.span()),
            }),
            ConstValue::Enum(val) => DeserValue::Enum(EnumValue {
                name: val.name(),
                span: Some(val.span()),
            }),
            ConstValue::List(val) => DeserValue::List(val.into()),
            ConstValue::Object(val) => DeserValue::Object(val.into()),
        }
    }
}
