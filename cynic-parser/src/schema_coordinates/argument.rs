use crate::Span;

use super::{MemberCoordinate, Name};

pub struct ArgumentCoordinate {
    pub(super) member: MemberCoordinate,
    pub(super) name: Name,
}

impl ArgumentCoordinate {
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
