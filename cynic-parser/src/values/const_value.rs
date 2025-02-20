use crate::{AstLookup, Span};

use super::{
    const_lists::ConstList,
    const_objects::ConstObject,
    enums::EnumValue,
    iter::{Iter, ValueStoreReader},
    scalars::{BooleanValue, FloatValue, IntValue, NullValue, StringValue},
    value::ValueKind,
    ConstObjectField, ConstValueId, Cursor, Value, ValueId,
};

#[derive(Debug, Copy, Clone)]
pub enum ConstValue<'a> {
    Int(IntValue<'a>),
    Float(FloatValue<'a>),
    String(StringValue<'a>),
    Boolean(BooleanValue<'a>),
    Null(NullValue<'a>),
    Enum(EnumValue<'a>),
    List(ConstList<'a>),
    Object(ConstObject<'a>),
}

impl ConstValue<'_> {
    pub fn span(&self) -> Span {
        match self {
            ConstValue::Int(inner) => inner.span(),
            ConstValue::Float(inner) => inner.span(),
            ConstValue::String(inner) => inner.span(),
            ConstValue::Boolean(inner) => inner.span(),
            ConstValue::Null(inner) => inner.span(),
            ConstValue::Enum(inner) => inner.span(),
            ConstValue::List(inner) => inner.span(),
            ConstValue::Object(inner) => inner.span(),
        }
    }
}

impl<'a> ConstValue<'a> {
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

    pub fn as_list(&self) -> Option<ConstList<'a>> {
        match self {
            Self::List(inner) => Some(*inner),
            _ => None,
        }
    }

    pub fn as_items(&self) -> Option<Iter<'a, ConstValue<'a>>> {
        match self {
            Self::List(inner) => Some(inner.items()),
            _ => None,
        }
    }

    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    pub fn as_object(&self) -> Option<ConstObject<'a>> {
        match self {
            Self::Object(inner) => Some(*inner),
            _ => None,
        }
    }

    pub fn as_fields(&self) -> Option<Iter<'a, ConstObjectField<'a>>> {
        match self {
            Self::Object(inner) => Some(inner.fields()),
            _ => None,
        }
    }
}

impl PartialEq for ConstValue<'_> {
    #[allow(clippy::cmp_owned)]
    fn eq(&self, other: &Self) -> bool {
        Value::from(*self) == Value::from(*other)
    }
}

impl PartialEq<Value<'_>> for ConstValue<'_> {
    #[allow(clippy::cmp_owned)]
    fn eq(&self, other: &Value<'_>) -> bool {
        Value::from(*self) == *other
    }
}

impl PartialEq<ConstValue<'_>> for Value<'_> {
    #[allow(clippy::cmp_owned)]
    fn eq(&self, other: &ConstValue<'_>) -> bool {
        *self == Value::from(*other)
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
            ValueKind::List(_) => ConstValue::List(ConstList(cursor)),
            ValueKind::Object(_) => ConstValue::Object(ConstObject(cursor)),
        }
    }
}

impl<'a> TryFrom<Value<'a>> for ConstValue<'a> {
    type Error = ();

    fn try_from(value: Value<'a>) -> Result<Self, Self::Error> {
        if !const_safe(value) {
            return Err(());
        }

        Ok(match value {
            Value::Variable(_) => unreachable!(),
            Value::Int(int_value) => ConstValue::Int(int_value),
            Value::Float(float_value) => ConstValue::Float(float_value),
            Value::String(string_value) => ConstValue::String(string_value),
            Value::Boolean(boolean_value) => ConstValue::Boolean(boolean_value),
            Value::Null(null_value) => ConstValue::Null(null_value),
            Value::Enum(enum_value) => ConstValue::Enum(enum_value),
            Value::List(list_value) => {
                let id = ConstValueId::new(list_value.0.id.get());
                ConstValue::List(ConstList(Cursor {
                    id,
                    store: list_value.0.store,
                }))
            }
            Value::Object(object) => {
                let id = ConstValueId::new(object.0.id.get());
                ConstValue::Object(ConstObject(Cursor {
                    id,
                    store: object.0.store,
                }))
            }
        })
    }
}

fn const_safe(value: Value<'_>) -> bool {
    match value {
        Value::Variable(_) => false,
        Value::Int(_)
        | Value::Float(_)
        | Value::String(_)
        | Value::Boolean(_)
        | Value::Null(_)
        | Value::Enum(_) => true,
        Value::List(items) => items.items().all(const_safe),
        Value::Object(object) => object.fields().all(|field| const_safe(field.value())),
    }
}
