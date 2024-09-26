use crate::{common::IdRange, AstLookup, Span};

use super::{
    enums::EnumValue,
    ids::{FieldId, ListItemId, StringId},
    iter::ValueStoreReader,
    lists::ListValue,
    objects::ObjectValue,
    scalars::{BooleanValue, FloatValue, IntValue, NullValue, StringValue},
    variables::VariableValue,
    ValueId,
};

pub enum Value<'a> {
    Variable(VariableValue<'a>),
    Int(IntValue<'a>),
    Float(FloatValue<'a>),
    String(StringValue<'a>),
    Boolean(BooleanValue<'a>),
    Null(NullValue<'a>),
    Enum(EnumValue<'a>),
    List(ListValue<'a>),
    Object(ObjectValue<'a>),
}

impl<'a> Value<'a> {
    pub fn span(&self) -> Span {
        todo!()
    }
}

impl<'a> ValueStoreReader<'a> for Value<'a> {
    type Id = ValueId;
}

impl super::ValueStoreId for ValueId {
    type Reader<'a> = Value<'a>;

    fn read(self, store: &super::ValueStore) -> Self::Reader<'_> {
        todo!("impl me")
    }
}

pub struct ValueRecord {
    pub span: Span,
    pub kind: ValueKind,
}

pub enum ValueKind {
    Variable(StringId),
    Int(i64),
    Float(f32),
    String(StringId),
    Boolean(bool),
    Null,
    Enum(StringId),
    List(IdRange<ListItemId>),
    Object(IdRange<FieldId>),
}

impl ValueKind {
    pub fn as_variable(&self) -> Option<StringId> {
        match self {
            ValueKind::Variable(string_id) => Some(*string_id),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            ValueKind::Int(int) => Some(*int),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f32> {
        match self {
            ValueKind::Float(inner) => Some(*inner),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<StringId> {
        match self {
            ValueKind::String(inner) => Some(*inner),
            _ => None,
        }
    }

    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            ValueKind::Boolean(inner) => Some(*inner),
            _ => None,
        }
    }

    pub fn as_null(&self) -> Option<()> {
        match self {
            ValueKind::Null => Some(()),
            _ => None,
        }
    }

    pub fn as_enum_value(&self) -> Option<StringId> {
        match self {
            ValueKind::Enum(inner) => Some(*inner),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<IdRange<ListItemId>> {
        match self {
            ValueKind::List(inner) => Some(*inner),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<IdRange<FieldId>> {
        match self {
            ValueKind::Object(inner) => Some(*inner),
            _ => None,
        }
    }
}
