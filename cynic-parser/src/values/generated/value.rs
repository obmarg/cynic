use super::prelude::*;
use super::{
    enums::EnumValue,
    ids::{
        BooleanValueId, EnumValueId, FloatValueId, IntValueId, ListValueId, NullValueId,
        ObjectValueId, StringValueId, ValueId, VariableValueId,
    },
    scalars::BooleanValue,
    scalars::FloatValue,
    scalars::IntValue,
    scalars::NullValue,
    scalars::StringValue,
    variables::VariableValue,
    ListValue::ListValue,
    ObjectValue::ObjectValue,
    ReadContext, ValueId,
};
#[allow(unused_imports)]
use std::fmt::{self, Write};

pub enum ValueRecord {
    VariableValue(VariableValueId),
    IntValue(IntValueId),
    FloatValue(FloatValueId),
    StringValue(StringValueId),
    BooleanValue(BooleanValueId),
    NullValue(NullValueId),
    EnumValue(EnumValueId),
    ListValue(ListValueId),
    ObjectValue(ObjectValueId),
}

#[derive(Clone, Copy, Debug)]
pub enum Value<'a> {
    VariableValue(VariableValue<'a>),
    IntValue(IntValue<'a>),
    FloatValue(FloatValue<'a>),
    StringValue(StringValue<'a>),
    BooleanValue(BooleanValue<'a>),
    NullValue(NullValue<'a>),
    EnumValue(EnumValue<'a>),
    ListValue(ListValue<'a>),
    ObjectValue(ObjectValue<'a>),
}

impl ValueId for ValueId {
    type Reader<'a> = Value<'a>;
}

impl IdReader for Value<'_> {
    type Id = ValueId;
}

impl<'a> From<ReadContext<'a, ValueId>> for Value<'a> {
    fn from(value: ReadContext<'a, ValueId>) -> Self {
        match value.document.lookup(value.id) {
            ValueRecord::VariableValue(id) => Value::VariableValue(value.document.read(*id)),
            ValueRecord::IntValue(id) => Value::IntValue(value.document.read(*id)),
            ValueRecord::FloatValue(id) => Value::FloatValue(value.document.read(*id)),
            ValueRecord::StringValue(id) => Value::StringValue(value.document.read(*id)),
            ValueRecord::BooleanValue(id) => Value::BooleanValue(value.document.read(*id)),
            ValueRecord::NullValue(id) => Value::NullValue(value.document.read(*id)),
            ValueRecord::EnumValue(id) => Value::EnumValue(value.document.read(*id)),
            ValueRecord::ListValue(id) => Value::ListValue(value.document.read(*id)),
            ValueRecord::ObjectValue(id) => Value::ObjectValue(value.document.read(*id)),
        }
    }
}