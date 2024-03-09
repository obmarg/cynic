use crate::common::TypeWrappers;

use super::ids::StringId;

pub struct TypeRecord {
    pub name: StringId,
    pub wrappers: TypeWrappers,
}
