use std::fmt;

use crate::{AstLookup, Span};

use super::{ids::ValueId, iter::Iter, value::Value};

pub struct ListValue<'a>(pub(super) super::Cursor<'a, ValueId>);

impl<'a> ListValue<'a> {
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        let store = &self.0.store;
        store.lookup(self.0.id).kind.as_list().unwrap().len()
    }

    pub fn span(&self) -> Span {
        let store = &self.0.store;
        store.lookup(self.0.id).span
    }

    pub fn items(&self) -> Iter<'a, Value<'a>> {
        let store = &self.0.store;
        Iter::new(store.lookup(self.0.id).kind.as_list().unwrap(), store)
    }
}

impl fmt::Debug for ListValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.items()).finish()
    }
}
