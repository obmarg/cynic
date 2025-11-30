use crate::Span;

use super::{Name, TypeCoordinate};

pub struct MemberCoordinate {
    pub(super) ty: TypeCoordinate,
    pub(super) name: Name,
}

impl MemberCoordinate {
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
