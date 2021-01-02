//! A mutation example using the GitHub API
//!
//! Note that a lot of this example is feature flagged: this is because rust-analyzer
//! wants to build it as part of the cynic package when I'm working on cynic.  It
//! builds quite slow because of the size of the GitHub API (the query_dsl output is
//! around 100k lines of rust), and it's too much for normal development.
//!
//! If you want to use this example be sure to remove all the feature flagging.
//!
//! This example also requires the `reqwest-blocking` feature to be active.

fn main() {
    #[cfg(feature = "github")]
    {
        let result = run_query();
        println!("{:?}", result);
    }
}

#[cfg(feature = "github")]
fn run_query() -> cynic::GraphQLResponse<queries::CommentOnMutationSupportIssue> {
    use cynic::http::ReqwestBlockingExt;

    let query = build_query();

    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env var must be set");

    reqwest::blocking::Client::new()
        .post("https://api.github.com/graphql")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "obmarg")
        .run_graphql(query)
        .unwrap()
}

#[cfg(feature = "github")]
fn build_query() -> cynic::Operation<'static, queries::CommentOnMutationSupportIssue> {
    use cynic::MutationBuilder;
    use queries::{CommentOnMutationSupportIssue, CommentOnMutationSupportIssueArguments};

    CommentOnMutationSupportIssue::build(&CommentOnMutationSupportIssueArguments {
        comment_body: "Testing".into(),
    })
}

#[cfg(feature = "github")]
#[cynic::query_module(schema_path = "../schemas/github.graphql", query_module = "query_dsl")]
mod queries {
    use super::query_dsl;

    #[derive(cynic::FragmentArguments, Debug)]
    pub struct CommentOnMutationSupportIssueArguments {
        pub comment_body: String,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(
        graphql_type = "Mutation",
        argument_struct = "CommentOnMutationSupportIssueArguments"
    )]
    pub struct CommentOnMutationSupportIssue {
        #[arguments(input = AddCommentInput {
            body: args.comment_body.clone(),
            subject_id: "MDU6SXNzdWU2ODU4NzUxMzQ=".into(),
            client_mutation_id: None
        })]
        pub add_comment: Option<AddCommentPayload>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "AddCommentPayload")]
    pub struct AddCommentPayload {
        pub comment_edge: Option<IssueCommentEdge>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "IssueCommentEdge")]
    pub struct IssueCommentEdge {
        pub node: Option<IssueComment>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "IssueComment")]
    pub struct IssueComment {
        pub id: cynic::Id,
    }

    #[derive(cynic::InputObject, Clone, Debug)]
    #[cynic(graphql_type = "AddCommentInput", rename_all = "camelCase")]
    pub struct AddCommentInput {
        pub body: String,
        pub client_mutation_id: Option<String>,
        pub subject_id: cynic::Id,
    }
}

#[cfg(feature = "github")]
#[cynic::query_module(schema_path = "../schemas/github.graphql", query_module = "query_dsl")]
mod types {
    #[derive(cynic::Scalar, Debug)]
    pub struct Date(String);

    #[derive(cynic::Scalar, Debug)]
    pub struct DateTime(String);

    #[derive(cynic::Scalar, Debug)]
    pub struct GitObjectID(String);

    #[derive(cynic::Scalar, Debug)]
    pub struct GitRefname(String);

    #[derive(cynic::Scalar, Debug)]
    pub struct GitSSHRemote(String);

    #[derive(cynic::Scalar, Debug)]
    pub struct GitTimestamp(String);

    #[derive(cynic::Scalar, Debug)]
    pub struct Html(String);

    #[derive(cynic::Scalar, Debug)]
    pub struct PreciseDateTime(String);

    #[derive(cynic::Scalar, Debug)]
    pub struct Uri(String);

    #[derive(cynic::Scalar, Debug)]
    pub struct X509Certificate(String);
}

#[cfg(feature = "github")]
mod query_dsl {
    use super::types::*;
    cynic::query_dsl!("../schemas/github.graphql");
}

#[cfg(all(test, feature = "github"))]
mod test {
    use super::*;

    #[test]
    fn snapshot_test_query() {
        // Running a snapshot test of the query building functionality as that gives us
        // a place to copy and paste the actual GQL we're using for running elsewhere,
        // and also helps ensure we don't change queries by mistake

        let query = build_query();

        insta::assert_snapshot!(query.query);
    }

    #[test]
    fn test_running_query() {
        let result = run_query();
        if result.errors.is_some() {
            assert_eq!(result.errors.unwrap().len(), 0);
        }
    }
}
