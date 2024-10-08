use super::iter::Iter;
use super::{ids, TypeSystemId};

/// A prelude module for all the generated modules
///
/// Anything in here will be pulled into scope for the modules
///
/// This makes the generator simpler as it doesn't need to dynamically
/// figure out how to import everything external it needs - it can just
/// `use prelude::*` and be done with it.
mod prelude {
    pub(super) use crate::{
        common::{IdRange, OperationType},
        type_system::{
            ids::StringId,
            iter::{IdReader, Iter},
            DirectiveLocation, ReadContext, TypeSystemDocument,
        },
        AstLookup, Span,
    };
}

pub mod value {
    pub use crate::values::ConstValue;
}

mod types {
    pub(super) use super::super::Type;
}

mod strings {
    pub(super) use super::super::StringLiteral;
}

pub mod arguments;
pub mod descriptions;
pub mod directives;
pub mod enums;
pub mod fields;
pub mod input_objects;
pub mod input_values;
pub mod interfaces;
pub mod objects;
pub mod scalars;
pub mod schemas;
pub mod unions;
