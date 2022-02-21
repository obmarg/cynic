//! An example using the GitHub API
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

fn run_query() -> cynic::GraphQlResponse<queries::PullRequestTitles> {
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

fn build_query() -> cynic::Operation<queries::PullRequestTitles, queries::PullRequestTitlesArguments>
{
    use cynic::QueryBuilder;
    use queries::{
        IssueOrder, IssueOrderField, OrderDirection, PullRequestTitles, PullRequestTitlesArguments,
    };

    PullRequestTitles::build(PullRequestTitlesArguments {
        pr_order: IssueOrder {
            direction: OrderDirection::Asc,
            field: IssueOrderField::CreatedAt,
        },
    })
}

#[cynic::schema_for_derives(file = "../schemas/github.graphql", module = "schema")]
mod queries {
    use github_schema as schema;

    pub type DateTime = chrono::DateTime<chrono::Utc>;

    #[derive(cynic::QueryVariables, Debug)]
    pub struct PullRequestTitlesArguments {
        pub pr_order: IssueOrder,
    }

    #[derive(cynic::InputObject, Clone, Debug)]
    #[cynic(rename_all = "camelCase")]
    pub struct IssueOrder {
        pub direction: OrderDirection,
        pub field: IssueOrderField,
    }

    #[derive(cynic::Enum, Clone, Copy, Debug)]
    #[cynic(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum OrderDirection {
        Asc,
        Desc,
    }

    #[derive(cynic::Enum, Clone, Copy, Debug)]
    #[cynic(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum IssueOrderField {
        Comments,
        CreatedAt,
        UpdatedAt,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Query", argument_struct = "PullRequestTitlesArguments")]
    pub struct PullRequestTitles {
        #[arguments(name = "cynic".to_string(), owner = "obmarg".to_string())]
        pub repository: Option<Repository>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(argument_struct = "PullRequestTitlesArguments")]
    pub struct Repository {
        #[arguments(orderBy: $pr_order, first: 10)]
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
        pub created_at: DateTime,
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
        assert_eq!(
            result
                .data
                .as_ref()
                .unwrap()
                .repository
                .as_ref()
                .unwrap()
                .pull_requests
                .nodes
                .len(),
            10
        );
        insta::assert_debug_snapshot!(result.data);
    }
}
