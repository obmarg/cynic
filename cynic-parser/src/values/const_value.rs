use crate::{AstLookup, Span};

use super::{
    const_lists::ConstListValue,
    const_objects::ConstObjectValue,
    enums::EnumValue,
    iter::{Iter, ValueStoreReader},
    scalars::{BooleanValue, FloatValue, IntValue, NullValue, StringValue},
    value::ValueKind,
    ConstValueId, Cursor, ValueId,
};

#[derive(Debug)]
pub enum ConstValue<'a> {
    Int(IntValue<'a>),
    Float(FloatValue<'a>),
    String(StringValue<'a>),
    Boolean(BooleanValue<'a>),
    Null(NullValue<'a>),
    Enum(EnumValue<'a>),
    List(ConstListValue<'a>),
    Object(ConstObjectValue<'a>),
}

impl<'a> ConstValue<'a> {
    pub fn span(&self) -> Span {
        todo!()
    }
}

impl<'a> ConstValue<'a> {
    pub fn as_str(&self) -> Option<&'a str> {
        match self {
            Self::String(inner) => Some(inner.value()),
            _ => None,
        }
    }

    pub fn as_list_iter(&self) -> Option<Iter<'a, ConstValue<'a>>> {
        match self {
            Self::List(inner) => Some(inner.items()),
            _ => None,
        }
    }
}

impl<'a> ValueStoreReader<'a> for ConstValue<'a> {
    type Id = ConstValueId;
}

impl super::ValueStoreId for ConstValueId {
    type Reader<'a> = ConstValue<'a>;

    fn read(self, store: &super::ValueStore) -> Self::Reader<'_> {
        let value_id = ValueId::from(self);
        let value_cursor = Cursor {
            id: value_id,
            store,
        };
        let cursor = Cursor { id: self, store };

        match store.lookup(value_id).kind {
            ValueKind::Variable(_) => unreachable!("variable found under ConstValueId"),
            ValueKind::Int(_) => ConstValue::Int(IntValue(value_cursor)),
            ValueKind::Float(_) => ConstValue::Float(FloatValue(value_cursor)),
            ValueKind::String(_) => ConstValue::String(StringValue(value_cursor)),
            ValueKind::Boolean(_) => ConstValue::Boolean(BooleanValue(value_cursor)),
            ValueKind::Null => ConstValue::Null(NullValue(value_cursor)),
            ValueKind::Enum(_) => ConstValue::Enum(EnumValue(value_cursor)),
            ValueKind::List(_) => ConstValue::List(ConstListValue(cursor)),
            ValueKind::Object(_) => ConstValue::Object(ConstObjectValue(cursor)),
        }
    }
}
