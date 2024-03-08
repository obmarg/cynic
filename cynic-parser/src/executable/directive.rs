use crate::common::IdRange;

use super::ids::{ArgumentId, StringId};

pub struct DirectiveRecord {
    pub name: StringId,
    pub arguments: IdRange<ArgumentId>,
}
