//! An example using the GitHub API
//!
//! Note that a lot of this example is feature flagged: this is because rust-analyzer
//! wants to build it as part of the cynic package when I'm working on cynic.  It
//! builds quite slow because of the size of the GitHub API (the schema output is
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

#[cfg(feature = "github")]
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

#[cfg(feature = "github")]
#[cynic::schema_for_derives(file = "../schemas/github.graphql", module = "schema")]
mod queries {
    use super::schema;

    pub type DateTime = chrono::DateTime<chrono::Utc>;

    cynic::impl_scalar!(DateTime, schema::DateTime);

    #[derive(cynic::FragmentArguments, Debug)]
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

#[cfg(feature = "github")]
mod schema {
    cynic::use_schema!("../schemas/github.graphql");
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
