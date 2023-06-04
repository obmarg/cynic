#![allow(unused_imports)]

fn main() {}

#[cynic::schema_for_derives(file = r#"./../../../../schemas/github.graphql"#, module = "schema")]
mod queries {
    use super::schema;

    #[derive(cynic::QueryFragment, Debug)]
    pub struct CheckSuite {
        // Note: this is the wrong underlying enum type
        // This field is actually a CheckStatusState, so this should fail to
        // compile
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

mod schema {
    cynic::use_schema!(r#"./../../../../schemas/github.graphql"#);
}
