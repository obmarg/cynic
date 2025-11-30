use crate::Span;

use super::Name;

pub struct TypeCoordinate {
    pub(super) name: Name,
}

impl TypeCoordinate {
    pub fn span(&self) -> Span {
        self.name.span
    }

    pub fn name(&self) -> &str {
        &self.name.value
    }
}
