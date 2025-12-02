use std::fmt;

use crate::Span;

use super::Name;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct TypeCoordinate {
    pub(super) name: Name,
}

impl TypeCoordinate {
    pub fn new(name: impl Into<String>) -> Self {
        TypeCoordinate {
            name: Name::new(name.into()),
        }
    }

    pub fn span(&self) -> Span {
        self.name.span
    }

    pub fn name(&self) -> &str {
        &self.name.value
    }
}

impl fmt::Display for TypeCoordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name.value)
    }
}
