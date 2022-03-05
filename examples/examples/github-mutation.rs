//! A mutation example using the GitHub API
//!
//! Note that this example pulls a schema from the `github_schema` crate.
//! This is because the github schema is massive and compiling the output
//! of `use_schema` is quite slow.  Moving this into a separate crate
//! means we won't have to recompile it every time this file changes
//! and we'll only need to do so once per full build of cynic.
//!
//! You may want to do similar if you're also working with cynic & the
//! github API.
//!
//! This example requires the `reqwest-blocking` feature to be active.

fn main() {
    let result = run_query();
    println!("{:?}", result);
}

fn run_query() -> cynic::GraphQlResponse<queries::CommentOnMutationSupportIssue> {
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

fn build_query() -> cynic::Operation<
    queries::CommentOnMutationSupportIssue,
    queries::CommentOnMutationSupportIssueArguments,
> {
    use cynic::MutationBuilder;
    use queries::{CommentOnMutationSupportIssue, CommentOnMutationSupportIssueArguments};

    CommentOnMutationSupportIssue::build(CommentOnMutationSupportIssueArguments {
        comment_body: "Testing".into(),
    })
}

#[cynic::schema_for_derives(file = "../schemas/github.graphql", module = "schema")]
mod queries {
    use github_schema as schema;

    #[derive(cynic::QueryVariables, Debug)]
    pub struct CommentOnMutationSupportIssueArguments {
        pub comment_body: String,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(
        graphql_type = "Mutation",
        variables = "CommentOnMutationSupportIssueArguments"
    )]
    pub struct CommentOnMutationSupportIssue {
        #[arguments(input: {
            body: $comment_body,
            subjectId: "MDU6SXNzdWU2ODU4NzUxMzQ=",
            clientMutationId: null,
        })]
        pub add_comment: Option<AddCommentPayload>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct AddCommentPayload {
        pub comment_edge: Option<IssueCommentEdge>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct IssueCommentEdge {
        pub node: Option<IssueComment>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct IssueComment {
        pub id: cynic::Id,
    }

    #[derive(cynic::InputObject, Clone, Debug)]
    #[cynic(rename_all = "camelCase")]
    pub struct AddCommentInput {
        pub body: String,
        pub client_mutation_id: Option<String>,
        pub subject_id: cynic::Id,
    }
}

#[cfg(test)]
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
    #[ignore]
    fn test_running_query() {
        let result = run_query();
        if result.errors.is_some() {
            assert_eq!(result.errors.unwrap().len(), 0);
        }
    }
}
