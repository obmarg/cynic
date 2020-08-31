//! An example using the GitHub API
//!
//! Note that a lot of this example is feature flagged: this is because rust-analyzer
//! wants to build it as part of the cynic package when I'm working on cynic.  It
//! builds quite slow because of the size of the GitHub API (the query_dsl output is
//! around 100k lines of rust), and it's too much for normal development.
//!
//! If you want to use this example be sure to remove all the feature flagging.

fn main() {
    #[cfg(feature = "github")]
    {
        let result = run_query();
        println!("{:?}", result);
    }
}

#[cfg(feature = "github")]
fn run_query() -> cynic::GraphQLResponse<queries::PullRequestTitles> {
    let query = build_query();

    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env var must be set");

    let response = reqwest::blocking::Client::new()
        .post("https://api.github.com/graphql")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "obmarg")
        .json(&query)
        .send()
        .unwrap();

    let response_body = response.text().unwrap();
    println!("{:?}", &response_body);

    query
        .decode_response(serde_json::from_str(&response_body).unwrap())
        .unwrap()
}

#[cfg(feature = "github")]
fn build_query() -> cynic::Operation<'static, queries::PullRequestTitles> {
    use cynic::QueryFragment;
    use queries::{
        IssueOrder, IssueOrderField, OrderDirection, PullRequestTitles, PullRequestTitlesArguments,
    };

    cynic::Operation::query(PullRequestTitles::fragment(PullRequestTitlesArguments {
        pr_order: IssueOrder {
            direction: OrderDirection::Asc,
            field: IssueOrderField::CreatedAt,
        },
    }))
}

#[cfg(feature = "github")]
#[cynic::query_module(
    schema_path = "../cynic-querygen/tests/schemas/github.graphql",
    query_module = "query_dsl"
)]
mod queries {
    use super::{query_dsl, types::*};

    #[derive(cynic::FragmentArguments, Clone, Debug)]
    pub struct PullRequestTitlesArguments {
        pub pr_order: IssueOrder,
    }

    #[derive(cynic::InputObject, Clone, Debug)]
    #[cynic(graphql_type = "IssueOrder", rename_all = "camelCase")]
    pub struct IssueOrder {
        pub direction: OrderDirection,
        pub field: IssueOrderField,
    }

    #[derive(cynic::Enum, Clone, Copy, Debug)]
    #[cynic(graphql_type = "OrderDirection", rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum OrderDirection {
        Asc,
        Desc,
    }

    #[derive(cynic::Enum, Clone, Copy, Debug)]
    #[cynic(graphql_type = "IssueOrderField", rename_all = "SCREAMING_SNAKE_CASE")]
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
    #[cynic(
        graphql_type = "Repository",
        argument_struct = "PullRequestTitlesArguments"
    )]
    pub struct Repository {
        #[arguments(order_by = &args.pr_order, first = 10)]
        pub pull_requests: PullRequestConnection,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "PullRequestConnection")]
    pub struct PullRequestConnection {
        #[cynic(flatten)]
        pub nodes: Vec<PullRequest>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "PullRequest")]
    pub struct PullRequest {
        pub title: String,
    }
}

#[cfg(feature = "github")]
#[cynic::query_module(
    schema_path = "../cynic-querygen/tests/schemas/github.graphql",
    query_module = "query_dsl"
)]
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
    cynic::query_dsl!("../cynic-querygen/tests/schemas/github.graphql");
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
