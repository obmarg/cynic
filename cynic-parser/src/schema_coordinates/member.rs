use std::fmt;

use crate::Span;

use super::{Name, TypeCoordinate};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct MemberCoordinate {
    pub(super) ty: TypeCoordinate,
    pub(super) name: Name,
}

impl MemberCoordinate {
    pub fn new(ty: impl Into<String>, field: impl Into<String>) -> Self {
        MemberCoordinate {
            ty: TypeCoordinate::new(ty),
            name: Name::new(field.into()),
        }
    }

    pub fn ty(&self) -> &TypeCoordinate {
        &self.ty
    }

    pub fn span(&self) -> Span {
        self.name.span
    }

    pub fn name(&self) -> &str {
        &self.name.value
    }
}

impl fmt::Display for MemberCoordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.ty.name.value, self.name.value)
    }
}
