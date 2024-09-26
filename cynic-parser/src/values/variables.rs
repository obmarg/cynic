use core::fmt;

use crate::{AstLookup, Span};

use super::{ids::ValueId, Cursor};

#[derive(Clone, Copy)]
pub struct VariableValue<'a>(pub(super) Cursor<'a, ValueId>);

impl<'a> VariableValue<'a> {
    pub fn name(&self) -> &'a str {
        let store = self.0.store;
        store.lookup(store.lookup(self.0.id).kind.as_variable().unwrap())
    }

    pub fn span(&self) -> Span {
        let store = self.0.store;
        store.lookup(self.0.id).span
    }
}

impl VariableValue<'_> {
    pub fn id(&self) -> ValueId {
        self.0.id
    }
}

impl fmt::Display for VariableValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${}", self.name())
    }
}

impl fmt::Debug for VariableValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IntValue")
            .field("value", &self.name())
            .field("span", &self.span())
            .finish()
    }
}
