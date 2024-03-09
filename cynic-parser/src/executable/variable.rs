use crate::common::IdRange;

use super::ids::{DirectiveId, StringId, TypeId, ValueId};

pub struct VariableDefinitionRecord {
    pub name: StringId,
    pub ty: TypeId,
    pub default_value: Option<ValueId>,
    pub directives: IdRange<DirectiveId>,
}
