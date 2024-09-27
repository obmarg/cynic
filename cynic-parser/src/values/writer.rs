use std::sync::Arc;

use indexmap::IndexSet;

use crate::{common::IdRange, Span};

pub use super::{ids::*, storage::*};

#[derive(Default)]
pub struct ValueWriter {
    values: Vec<ValueRecord>,

    fields: Vec<FieldRecord>,
}

impl ValueWriter {
    pub fn update(store: super::ValueStore) -> Self {
        let super::ValueStore {
            strings: _,
            values,
            fields,
        } = store;

        Self { values, fields }
    }

    pub fn value(&mut self, record: ValueRecord) -> ValueId {
        let id = ValueId::new(self.values.len());
        self.values.push(record);
        id
    }

    pub fn const_value(&mut self, record: ValueRecord) -> ConstValueId {
        if let ValueKind::Variable(_) = record.kind {
            panic!("Don't pass a variable into const_value");
        }
        let value_id = self.value(record);
        ConstValueId::new(value_id.get())
    }

    pub fn list(&mut self, values: Vec<ValueRecord>) -> IdRange<ValueId> {
        let start = ValueId::new(self.values.len());

        self.values.extend(values);

        let end = ValueId::new(self.values.len());

        IdRange::new(start, end)
    }

    pub fn fields(&mut self, records: Vec<(StringId, Span, ValueId)>) -> IdRange<FieldId> {
        let start = FieldId::new(self.fields.len());

        self.fields.extend(
            records
                .into_iter()
                .map(|(name, name_span, value)| FieldRecord {
                    name,
                    name_span,
                    value,
                }),
        );

        let end = FieldId::new(self.fields.len());

        IdRange::new(start, end)
    }

    pub(crate) fn finish(self, strings: Arc<IndexSet<Box<str>>>) -> super::ValueStore {
        let ValueWriter { values, fields } = self;

        super::ValueStore {
            strings,
            values,
            fields,
        }
    }
}
