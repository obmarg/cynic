/// Derive, cache and export query structs in this crate 
/// to avoid potentially expensive derivation as per https://cynic-rs.dev/large-apis

use cynic; // Import for derive macros
use schema::github as schema; // Rename is vital! Must import as schema!


#[derive(cynic::QueryVariables, Debug)]
pub struct PullRequestTitlesArguments {
    pub pr_order: IssueOrder,
}

#[derive(cynic::InputObject, Clone, Debug)]
#[cynic(schema = "github", rename_all = "camelCase")]
pub struct IssueOrder {
    pub direction: OrderDirection,
    pub field: IssueOrderField,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(schema = "github", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderDirection {
    Asc,
    Desc,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
#[cynic(schema = "github", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IssueOrderField {
    Comments,
    CreatedAt,
    UpdatedAt,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    graphql_type = "Query",
    schema = "github",
    variables = "PullRequestTitlesArguments"
)]
pub struct PullRequestTitles {
    #[arguments(name = "cynic".to_string(), owner = "obmarg".to_string())]
    pub repository: Option<Repository>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema = "github", variables = "PullRequestTitlesArguments")]
pub struct Repository {
    #[arguments(orderBy: $pr_order, first: 10)]
    pub pull_requests: PullRequestConnection,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema = "github")]
pub struct PullRequestConnection {
    #[cynic(flatten)]
    pub nodes: Vec<PullRequest>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema = "github")]
pub struct PullRequest {
    pub title: String,
}
