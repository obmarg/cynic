use std::fmt;

use crate::Span;

use super::{MemberCoordinate, Name};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ArgumentCoordinate {
    pub(super) member: MemberCoordinate,
    pub(super) name: Name,
}

impl ArgumentCoordinate {
    pub fn new(
        ty: impl Into<String>,
        field: impl Into<String>,
        argument: impl Into<String>,
    ) -> Self {
        Self {
            member: MemberCoordinate::new(ty, field),
            name: Name::new(argument.into()),
        }
    }

    pub fn member(&self) -> &MemberCoordinate {
        &self.member
    }

    pub fn span(&self) -> Span {
        self.name.span
    }

    pub fn name(&self) -> &str {
        &self.name.value
    }
}

impl fmt::Display for ArgumentCoordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({}:)", self.member, self.name.value)
    }
}
