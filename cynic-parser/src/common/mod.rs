mod id_range;

pub use id_range::IdRange;

#[derive(Clone, Copy, Debug)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

impl std::fmt::Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            OperationType::Query => write!(f, "query"),
            OperationType::Mutation => write!(f, "mutation"),
            OperationType::Subscription => write!(f, "subscription"),
        }
    }
}
