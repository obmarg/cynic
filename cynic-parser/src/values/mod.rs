// mod store;
// mod generated;
mod enums;
mod lists;
mod objects;
mod scalars;
mod value;
mod variables;

pub mod ids;
pub mod iter;

use std::sync::Arc;

pub use self::{
    enums::EnumValue,
    lists::ListValue,
    objects::{FieldRecord, ObjectField, ObjectValue},
    scalars::{BooleanValue, FloatValue, IntValue, NullValue, StringValue},
    value::Value,
};

use ids::*;
use indexmap::IndexSet;

pub mod storage {
    pub use super::{objects::FieldRecord, value::ValueRecord};
}

pub struct ValueStore {
    // TODO: Figure out if this arc approach works ok...
    strings: Arc<IndexSet<Box<str>>>,

    values: Vec<storage::ValueRecord>,

    // TODO: Can this just be incorporated straight into values?
    // maybe...
    list_items: Vec<ValueId>,

    fields: Vec<storage::FieldRecord>,
}

impl ValueStore {
    pub fn read<T>(&self, id: T) -> T::Reader<'_>
    where
        T: ValueStoreId,
    {
        id.read(self)
    }
}

#[derive(Clone, Copy)]
pub struct Cursor<'a, I> {
    id: I,
    store: &'a ValueStore,
}

pub trait ValueStoreId: Copy {
    type Reader<'a>;

    fn read(self, store: &ValueStore) -> Self::Reader<'_>;
}
