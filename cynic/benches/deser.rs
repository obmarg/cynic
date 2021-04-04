use criterion::{criterion_group, criterion_main, Criterion};

use cynic::GraphQlResponse;
use std::{fs, io::Read};

fn criterion_benchmark(c: &mut Criterion) {
    let json = include_str!("deser-data.json");
    c.bench_function("direct_deserialization", |b| {
        b.iter_with_large_drop(|| serde_json::from_str::<GraphQlResponse<pr_query::PRs>>(json))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

#[cynic::schema_for_derives(file = r#"../schemas/github.graphql"#)]
mod pr_query {
    use super::schema;

    #[derive(cynic::QueryVariables, Debug)]
    pub struct PRsArguments {
        pub repo_name: String,
        pub repo_owner: String,
        pub pr_cursor: Option<String>,
        pub page_size: i32,
    }

    /// ```graphql
    /// query PRs($repoName: String!, $repoOwner: String!, $prCursor: String) {
    ///   repository(name: $repoName, owner: $repoOwner) {
    ///     pullRequests(first: 100, states: MERGED, after: $prCursor) {
    ///       pageInfo {
    ///         endCursor
    ///         hasNextPage
    ///       }
    ///       nodes {
    ///         commits(first: 250) {
    ///           nodes {
    ///             commit {
    ///               messageHeadline
    ///               authoredDate
    ///             }
    ///           }
    ///         }
    ///         mergeCommit {
    ///           messageHeadline
    ///           authoredDate
    ///           checkSuites(first: 25) {
    ///             nodes {
    ///               status
    ///               conclusion
    ///               updatedAt
    ///             }
    ///           }
    ///           status {
    ///             state
    ///             contexts {
    ///               state
    ///               createdAt
    ///             }
    ///           }
    ///         }
    ///       }
    ///     }
    ///   }
    /// }
    /// ```
    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Query", argument_struct = "PRsArguments")]
    pub struct PRs {
        #[arguments(name: $repo_name, owner: $repo_owner)]
        pub repository: Option<Repository>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(argument_struct = "PRsArguments")]
    pub struct Repository {
        #[arguments(first: $page_size, states = Some(vec![PullRequestState::Merged]), after: $pr_cursor)]
        pub pull_requests: PullRequestConnection,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct PullRequestConnection {
        pub page_info: PageInfo,
        pub total_count: i32,
        #[cynic(flatten)]
        pub nodes: Vec<PullRequest>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct PullRequest {
        #[arguments(first = 250)]
        pub commits: PullRequestCommitConnection,
        pub merge_commit: Option<MergeCommit>,
        pub author: Option<Actor>,
    }

    #[derive(cynic::InlineFragments, Debug)]
    pub enum Actor {
        User(User),

        #[cynic(fallback)]
        Other,
    }

    impl Actor {
        pub fn login(&self) -> Option<&str> {
            match self {
                Actor::User(user) => Some(&user.login),
                _ => None,
            }
        }
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct PullRequestCommitConnection {
        #[cynic(flatten)]
        pub nodes: Vec<PullRequestCommit>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct PullRequestCommit {
        pub commit: Commit,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct PageInfo {
        pub end_cursor: Option<String>,
        pub has_next_page: bool,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Commit")]
    pub struct MergeCommit {
        pub message_headline: String,
        pub authored_date: DateTime,
        #[arguments(first = 25)]
        pub check_suites: Option<CheckSuiteConnection>,
        pub status: Option<Status>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Commit {
        pub message_headline: String,
        pub authored_date: DateTime,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct CheckSuiteConnection {
        #[cynic(flatten)]
        pub nodes: Vec<CheckSuite>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct CheckSuite {
        pub status: CheckStatusState,
        pub conclusion: Option<CheckConclusionState>,
        pub updated_at: DateTime,
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

    #[derive(cynic::Enum, Clone, Copy, Debug, PartialEq)]
    pub enum CheckStatusState {
        Completed,
        InProgress,
        Queued,
        Requested,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct Status {
        pub state: StatusState,
        pub contexts: Vec<StatusContext>,
    }

    #[derive(cynic::Enum, Clone, Copy, Debug, PartialEq)]
    pub enum StatusState {
        Error,
        Expected,
        Failure,
        Pending,
        Success,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct StatusContext {
        pub created_at: DateTime,
    }

    #[derive(cynic::Enum, Clone, Copy, Debug, PartialEq)]
    pub enum PullRequestState {
        Closed,
        Merged,
        Open,
    }

    #[derive(cynic::QueryFragment, Debug)]
    pub struct User {
        pub login: String,
    }

    #[derive(cynic::Scalar, Debug, Clone)]
    pub struct DateTime(pub chrono::DateTime<chrono::Utc>);
}

mod schema {
    cynic::use_schema!(r#"../schemas/github.graphql"#);
}
