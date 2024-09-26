use crate::{AstLookup, Span};

use super::{
    ids::ValueId,
    iter::{Iter, ValueStoreReader},
    value::Value,
};

pub struct ListValue<'a>(super::Cursor<'a, ValueId>);

impl<'a> ListValue<'a> {
    pub fn span(&self) -> Span {
        let store = &self.0.store;
        store.lookup(self.0.id).span
    }

    pub fn items(&self) -> Iter<'a, Value<'a>> {
        todo!()
    }
}
