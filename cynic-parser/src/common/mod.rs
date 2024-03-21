mod id_range;
mod strings;
mod types;

pub use id_range::IdRange;
pub use strings::MalformedStringError;
pub use types::*;

pub(crate) use strings::{trim_block_string_whitespace, unquote_block_string, unquote_string};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

impl OperationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            OperationType::Query => "query",
            OperationType::Mutation => "mutation",
            OperationType::Subscription => "subscription",
        }
    }
}

impl std::fmt::Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
