use super::{
    ids::{self, ValueId},
    Cursor as ReadContext,
};

pub mod enums;
pub mod names;
pub mod scalars;
pub mod value;
pub mod variables;

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
        values::{
            ids::StringId,
            iter::{IdReader, Iter},
        },
        AstLookup, Span,
    };
}
