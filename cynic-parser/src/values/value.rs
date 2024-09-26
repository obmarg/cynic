use crate::{common::IdRange, AstLookup, Span};

use super::{
    enums::EnumValue,
    ids::{FieldId, StringId},
    iter::ValueStoreReader,
    lists::ListValue,
    objects::ObjectValue,
    scalars::{BooleanValue, FloatValue, IntValue, NullValue, StringValue},
    variables::VariableValue,
    Cursor, ValueId,
};

#[derive(Debug)]
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
        let cursor = Cursor { id: self, store };

        match store.lookup(self).kind {
            ValueKind::Variable(_) => Value::Variable(VariableValue(cursor)),
            ValueKind::Int(_) => Value::Int(IntValue(cursor)),
            ValueKind::Float(_) => Value::Float(FloatValue(cursor)),
            ValueKind::String(_) => Value::String(StringValue(cursor)),
            ValueKind::Boolean(_) => Value::Boolean(BooleanValue(cursor)),
            ValueKind::Null => Value::Null(NullValue(cursor)),
            ValueKind::Enum(_) => Value::Enum(EnumValue(cursor)),
            ValueKind::List(_) => Value::List(ListValue(cursor)),
            ValueKind::Object(_) => Value::Object(ObjectValue(cursor)),
        }
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
    List(IdRange<ValueId>),
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

    pub fn as_list(&self) -> Option<IdRange<ValueId>> {
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

#[cfg(test)]
mod tests {
    use super::ValueRecord;

    #[test]
    fn test_size_of_record() {
        assert_eq!(std::mem::size_of::<ValueRecord>(), 32);
    }
}
