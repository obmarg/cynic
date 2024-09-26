use std::sync::Arc;

use crate::{common::IdRange, Span};

pub use super::{ids::*, storage::*};

#[derive(Default)]
pub struct ValueWriter {
    values: Vec<ValueRecord>,

    // TODO: Can this just be incorporated straight into values?
    // maybe...
    list_items: Vec<ValueId>,

    fields: Vec<FieldRecord>,
}

impl ValueWriter {
    pub fn value(&mut self, record: ValueRecord) -> ValueId {
        let id = ValueId::new(self.values.len());
        self.values.push(record);
        id
    }

    pub fn list_items(&mut self, values: Vec<ValueId>) -> IdRange<ListItemId> {
        todo!()
    }

    pub fn fields(&mut self, records: Vec<(StringId, Span, ValueId)>) -> IdRange<FieldId> {
        todo!()
    }

    pub(crate) fn finish(self, strings: Arc<indexmap::IndexSet<Box<str>>>) -> super::ValueStore {
        let ValueWriter {
            values,
            list_items,
            fields,
        } = self;

        super::ValueStore {
            strings,
            values,
            list_items,
            fields,
        }
    }
}
