enum ValueRecord {
    Variable(VariableValueRecordStringId),
    Int(i64),
    Float(f32),
    String(StringId),
    Boolean(bool),
    Null,
    Enum(StringId),

    // TODO: Figure out how to express these as IdRange
    // or similar.
    List(Vec<ValueId>),
    Object(Vec<(StringId, ValueId)>),
}

pub struct VariableValueRecord {
    name: StringId,
    span: Span,
}

pub enum Value<'a> {
    Variable(VariableValue),
    Int(IntValue),
    Float(FloatValue),
    String(StringValue),
    Boolean(BooleanValue),
    Null(NullValue),
    Enum(EnumValue),
    List(ListValue),
    Object(ObjectValue),
}

pub struct VariableValue {}
