use super::{ids, iter::Iter, types, ExecutableId, ReadContext};

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
        executable::{
            ids::StringId,
            iter::{IdReader, Iter},
            ExecutableDocument,
        },
        AstLookup,
    };
}

pub mod argument;
pub mod definition;
pub mod directive;
pub mod fragment;
pub mod operation;
pub mod selections;
pub mod variable;
