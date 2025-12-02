use core::fmt;

use crate::Span;

use super::Name;

#[derive(Clone, Debug, Eq)]
pub struct DirectiveCoordinate {
    pub(super) name: Name,
    pub(super) span: Span,
}

impl PartialEq for DirectiveCoordinate {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl std::hash::Hash for DirectiveCoordinate {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialOrd for DirectiveCoordinate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DirectiveCoordinate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl DirectiveCoordinate {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: Name::new(name.into()),
            span: Span::default(),
        }
    }

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

impl fmt::Display for DirectiveCoordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "@{}", self.name.value)
    }
}
