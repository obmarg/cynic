use crate::{AstLookup, Span};

use super::{ids::ValueId, Cursor};

#[derive(Clone, Copy)]
pub struct EnumValue<'a>(pub(in super::super) Cursor<'a, ValueId>);

impl<'a> EnumValue<'a> {
    fn name(&self) -> &'a str {
        let store = self.0.store;
        store.lookup(store.lookup(self.0.id).kind.as_enum_value().unwrap())
    }

    pub fn span(&self) -> Span {
        let store = self.0.store;
        store.lookup(self.0.id).span
    }
}

impl EnumValue<'_> {
    pub fn id(&self) -> ValueId {
        self.0.id
    }
}

impl std::fmt::Debug for EnumValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl std::fmt::Display for EnumValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
