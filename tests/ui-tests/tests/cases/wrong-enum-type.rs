#![allow(unused_imports)]

fn main() {}

#[cynic::query_module(
    schema_path = r#"./../../../schemas/github.graphql"#,
    query_module = "query_dsl"
)]
mod queries {
    use super::{query_dsl, types::*};

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "CheckSuite")]
    pub struct CheckSuite {
        // Note: this is the wrong underlying enum type
        // Should be CheckStatusState
        pub status: CheckConclusionState,
        pub conclusion: Option<CheckConclusionState>,
    }

    #[derive(cynic::Enum, Clone, Copy, Debug)]
    #[cynic(graphql_type = "CheckConclusionState")]
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

#[cynic::query_module(
    schema_path = r#"./../../../schemas/github.graphql"#,
    query_module = "query_dsl"
)]
mod types {
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

mod query_dsl {
    use super::types::*;
    cynic::query_dsl!(r#"./../../../schemas/github.graphql"#);
}
