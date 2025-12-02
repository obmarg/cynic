use core::fmt;

use crate::Span;

use super::{DirectiveCoordinate, Name};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct DirectiveArgumentCoordinate {
    pub(super) directive: DirectiveCoordinate,
    pub(super) name: Name,
}

impl DirectiveArgumentCoordinate {
    pub(crate) fn new(name: impl Into<String>, argument: impl Into<String>) -> Self {
        Self {
            directive: DirectiveCoordinate::new(name),
            name: Name::new(argument.into()),
        }
    }

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

impl fmt::Display for DirectiveArgumentCoordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({}:)", self.directive, self.name.value)
    }
}
