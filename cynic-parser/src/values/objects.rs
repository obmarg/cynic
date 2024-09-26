use crate::{AstLookup, Span};

use super::{
    ids::{FieldId, StringId, ValueId},
    iter::{Iter, ValueStoreReader},
    value::Value,
    ValueStoreId,
};

pub struct FieldRecord {
    name: StringId,
    name_span: Span,
    value: ValueId,
}

pub struct ObjectValue<'a>(super::Cursor<'a, ValueId>);

impl<'a> ObjectValue<'a> {
    pub fn span(&self) -> Span {
        let store = &self.0.store;
        store.lookup(self.0.id).span
    }

    pub fn fields(&self) -> Iter<'a, ObjectField<'a>> {
        let store = self.0.store;
        Iter::new(store.lookup(self.0.id).kind.as_object().unwrap(), store)
    }
}

// TODO: Make sure this ObjectField name has no obvious clashes
// and rename if it does
pub struct ObjectField<'a>(super::Cursor<'a, FieldId>);

impl<'a> ObjectField<'a> {
    pub fn span(&self) -> Span {
        todo!()
    }

    pub fn name(&self) -> &'a str {
        todo!()
    }

    pub fn value(&self) -> Value<'a> {
        todo!()
    }
}

impl<'a> ValueStoreReader<'a> for ObjectField<'a> {
    type Id = FieldId;
}

impl ValueStoreId for FieldId {
    type Reader<'a> = ObjectField<'a>;

    fn read(self, store: &super::ValueStore) -> Self::Reader<'_> {
        ObjectField(super::Cursor { id: self, store })
    }
}
