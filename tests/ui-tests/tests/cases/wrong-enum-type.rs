#![allow(unused_imports)]

fn main() {}

#[cynic::schema_for_derives(file = r#"./../../../schemas/github.graphql"#, module = "schema")]
mod queries {
    use super::{schema, types::*};

    #[derive(cynic::QueryFragment, Debug)]
    pub struct CheckSuite {
        // Note: this is the wrong underlying enum type
        // Should be CheckStatusState
        pub status: CheckConclusionState,
        pub conclusion: Option<CheckConclusionState>,
    }

    #[derive(cynic::Enum, Clone, Copy, Debug)]
    pub enum CheckConclusionState {
        ActionRequired,
        Cancelled,
        Failure,
        Neutral,
        Skipped,
        Stale,
        Success,
        TimedOut,
    }
}

#[cynic::schema_for_derives(file = r#"./../../../schemas/github.graphql"#, module = "schema")]
mod types {
    use super::schema;

    #[derive(cynic::Scalar, Debug, Clone)]
    pub struct Date(pub String);

    #[derive(cynic::Scalar, Debug, Clone)]
    pub struct DateTime(pub String);

    #[derive(cynic::Scalar, Debug, Clone)]
    pub struct GitObjectID(pub String);

    #[derive(cynic::Scalar, Debug, Clone)]
    pub struct GitRefname(pub String);

    #[derive(cynic::Scalar, Debug, Clone)]
    pub struct GitSSHRemote(pub String);

    #[derive(cynic::Scalar, Debug, Clone)]
    pub struct GitTimestamp(pub String);

    #[derive(cynic::Scalar, Debug, Clone)]
    pub struct Html(pub String);

    #[derive(cynic::Scalar, Debug, Clone)]
    pub struct PreciseDateTime(pub String);

    #[derive(cynic::Scalar, Debug, Clone)]
    pub struct Uri(pub String);

    #[derive(cynic::Scalar, Debug, Clone)]
    pub struct X509Certificate(pub String);
}

mod schema {
    use super::types::*;
    cynic::use_schema!(r#"./../../../schemas/github.graphql"#);
}
