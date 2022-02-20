#![allow(unused_imports)]

fn main() {}

#[cynic::schema_for_derives(file = r#"./../../../schemas/github.graphql"#, module = "schema")]
mod queries {
    use super::schema;

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(argument_struct = "PullRequestTitlesArguments")]
    pub struct Repository {
        #[arguments(order_by: "COMMENTS", first: 10)]
        pub pull_requests: PullRequestConnection,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct PullRequestConnection {
        #[cynic(flatten)]
        pub nodes: Vec<PullRequest>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct PullRequest {
        pub title: String,
    }
}

mod schema {
    cynic::use_schema!(r#"./../../../schemas/github.graphql"#);
}
