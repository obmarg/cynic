use crate::Span;

use super::{DirectiveCoordinate, Name};

pub struct DirectiveArgumentCoordinate {
    pub(super) directive: DirectiveCoordinate,
    pub(super) name: Name,
}

impl DirectiveArgumentCoordinate {
    pub fn directive(&self) -> &DirectiveCoordinate {
        &self.directive
    }

    pub fn span(&self) -> Span {
        self.name.span
    }

    pub fn name(&self) -> &str {
        &self.name.value
    }
}
