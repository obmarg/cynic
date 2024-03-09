use super::ids::{StringId, ValueId};

pub enum ValueRecord {
    Variable(StringId),
    Int(i32),
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
