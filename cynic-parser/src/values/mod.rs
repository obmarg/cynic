// mod store;
mod generated;
mod ids;
mod iter;

use ids::{FieldId, ListItemId, StringId, ValueId};

use crate::{common::IdRange, Span};

pub mod storage {
    pub use super::generated::{enums::EnumValueRecord, value::ValueRecord};
}

pub struct ValueStore {
    values: Vec<(storage::ValueRecord, Span)>,

    list_items: Vec<ValueId>,

    fields: Vec<FieldRecord>,
}

pub struct FieldRecord {
    name: StringId,
    name_span: Span,
    value: ValueId,
}

pub enum ValueRecord {
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

#[derive(Clone, Copy)]
pub struct Cursor<'a, I> {
    id: I,
    document: &'a ValueStore,
}

pub trait ValueStoreId: Copy {
    type Reader<'a>;

    fn read(self, store: &ValueStore) -> Self::Reader<'_>;
}
