use crate::Span;

use super::Name;

pub struct DirectiveCoordinate {
    pub(super) name: Name,
    pub(super) span: Span,
}

impl DirectiveCoordinate {
    pub fn span(&self) -> Span {
        self.span
    }

    pub fn name_span(&self) -> Span {
        self.name.span
    }

    pub fn name(&self) -> &str {
        &self.name.value
    }
}
