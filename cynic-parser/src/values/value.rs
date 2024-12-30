use crate::{common::IdRange, AstLookup, Span};

use super::{
    const_value::ConstValue,
    enums::EnumValue,
    ids::{FieldId, StringId},
    iter::{Iter, ValueStoreReader},
    lists::List,
    objects::Object,
    scalars::{BooleanValue, FloatValue, IntValue, NullValue, StringValue},
    variables::VariableValue,
    Cursor, ObjectField, ValueId,
};

#[derive(Debug, Clone, Copy)]
pub enum Value<'a> {
    Variable(VariableValue<'a>),
    Int(IntValue<'a>),
    Float(FloatValue<'a>),
    String(StringValue<'a>),
    Boolean(BooleanValue<'a>),
    Null(NullValue<'a>),
    Enum(EnumValue<'a>),
    List(List<'a>),
    Object(Object<'a>),
}

impl<'a> Value<'a> {
    pub fn span(&self) -> Span {
        match self {
            Value::Variable(inner) => inner.span(),
            Value::Int(inner) => inner.span(),
            Value::Float(inner) => inner.span(),
            Value::String(inner) => inner.span(),
            Value::Boolean(inner) => inner.span(),
            Value::Null(inner) => inner.span(),
            Value::Enum(inner) => inner.span(),
            Value::List(inner) => inner.span(),
            Value::Object(inner) => inner.span(),
        }
    }

    /// Returns an iterator over all the variables that appear somewhere in this Value.
    ///
    /// Note that this is not deduplicated - if a variable appears more than once in
    /// the value it'll appear more than once in this iterator.
    pub fn variables_used(&self) -> impl Iterator<Item = &'a str> + '_ {
        VariableIterator {
            value_stack: vec![*self],
        }
    }
}

impl<'a> Value<'a> {
    pub fn is_variable(&self) -> bool {
        matches!(self, Value::Variable(_))
    }

    pub fn as_variable(&self) -> Option<&'a str> {
        match self {
            Self::Variable(inner) => Some(inner.name()),
            _ => None,
        }
    }

    pub fn is_int(&self) -> bool {
        matches!(self, Value::Int(_))
    }

    pub fn as_i32(&self) -> Option<i32> {
        match self {
            Self::Int(inner) => Some(inner.as_i32()),
            _ => None,
        }
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Value::Float(_))
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Float(inner) => Some(inner.value()),
            _ => None,
        }
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    pub fn as_str(&self) -> Option<&'a str> {
        match self {
            Self::String(inner) => Some(inner.value()),
            _ => None,
        }
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self, Value::Boolean(_))
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Boolean(boolean_value) => Some(boolean_value.value()),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null(_))
    }

    pub fn as_null(&self) -> Option<()> {
        match self {
            Value::Null(_) => Some(()),
            _ => None,
        }
    }

    pub fn is_enum(&self) -> bool {
        matches!(self, Value::Enum(_))
    }

    pub fn as_enum_value(&self) -> Option<&'a str> {
        match self {
            Value::Enum(value) => Some(value.name()),
            _ => None,
        }
    }

    pub fn is_list(&self) -> bool {
        matches!(self, Value::List(_))
    }

    pub fn as_list(&self) -> Option<List<'a>> {
        match self {
            Self::List(inner) => Some(*inner),
            _ => None,
        }
    }

    pub fn as_items(&self) -> Option<Iter<'a, Value<'a>>> {
        match self {
            Self::List(inner) => Some(inner.items()),
            _ => None,
        }
    }

    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }

    pub fn as_object(&self) -> Option<Object<'a>> {
        match self {
            Self::Object(inner) => Some(*inner),
            _ => None,
        }
    }

    pub fn as_fields(&self) -> Option<Iter<'a, ObjectField<'a>>> {
        match self {
            Self::Object(inner) => Some(inner.fields()),
            _ => None,
        }
    }
}

impl PartialEq for Value<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Variable(a), Value::Variable(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Null(_), Value::Null(_)) => true,
            (Value::Enum(a), Value::Enum(b)) => a == b,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => a == b,
            _ => false,
        }
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
            ValueKind::List(_) => Value::List(List(cursor)),
            ValueKind::Object(_) => Value::Object(Object(cursor)),
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
    Float(f64),
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

    pub fn as_float(&self) -> Option<f64> {
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

pub struct VariableIterator<'a> {
    value_stack: Vec<Value<'a>>,
}

impl<'a> Iterator for VariableIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(value) = self.value_stack.pop() {
            match value {
                Value::Object(fields) => self
                    .value_stack
                    .extend(fields.fields().map(|field| field.value())),
                Value::List(values) => self.value_stack.extend(values.items()),
                Value::Variable(variable) => return Some(variable.name()),
                _ => {}
            }
        }
        None
    }
}

impl<'a> From<ConstValue<'a>> for Value<'a> {
    fn from(value: ConstValue<'a>) -> Self {
        match value {
            ConstValue::Int(inner) => Value::Int(inner),
            ConstValue::Float(inner) => Value::Float(inner),
            ConstValue::String(inner) => Value::String(inner),
            ConstValue::Boolean(inner) => Value::Boolean(inner),
            ConstValue::Null(inner) => Value::Null(inner),
            ConstValue::Enum(inner) => Value::Enum(inner),
            ConstValue::List(inner) => Value::List(inner.into()),
            ConstValue::Object(inner) => Value::Object(inner.into()),
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
