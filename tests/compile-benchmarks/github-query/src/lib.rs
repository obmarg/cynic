pub use pr_query::*;
pub use team_query::*;

#[cynic::schema_for_derives(file = r#"src/github.schema.graphql"#)]
mod pr_query {
    use super::{schema, User};

    #[derive(cynic::FragmentArguments, Debug)]
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
        #[arguments(name = args.repo_name.clone(), owner = args.repo_owner.clone())]
        pub repository: Option<Repository>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Repository", argument_struct = "PRsArguments")]
    pub struct Repository {
        #[arguments(first = args.page_size, states = Some(vec![PullRequestState::Merged]), after = &args.pr_cursor)]
        pub pull_requests: PullRequestConnection,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "PullRequestConnection")]
    pub struct PullRequestConnection {
        pub page_info: PageInfo,
        pub total_count: i32,
        #[cynic(flatten)]
        pub nodes: Vec<PullRequest>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "PullRequest")]
    pub struct PullRequest {
        #[arguments(first = 250)]
        pub commits: PullRequestCommitConnection,
        pub merge_commit: Option<MergeCommit>,
        pub author: Option<Actor>,
    }

    #[derive(cynic::InlineFragments, Debug)]
    #[cynic(graphql_type = "Actor")]
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
    #[cynic(graphql_type = "PullRequestCommitConnection")]
    pub struct PullRequestCommitConnection {
        #[cynic(flatten)]
        pub nodes: Vec<PullRequestCommit>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "PullRequestCommit")]
    pub struct PullRequestCommit {
        pub commit: Commit,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "PageInfo")]
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
    #[cynic(graphql_type = "Commit")]
    pub struct Commit {
        pub message_headline: String,
        pub authored_date: DateTime,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "CheckSuiteConnection")]
    pub struct CheckSuiteConnection {
        #[cynic(flatten)]
        pub nodes: Vec<CheckSuite>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "CheckSuite")]
    pub struct CheckSuite {
        pub status: CheckStatusState,
        pub conclusion: Option<CheckConclusionState>,
        pub updated_at: DateTime,
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

    #[derive(cynic::Enum, Clone, Copy, Debug, PartialEq)]
    #[cynic(graphql_type = "CheckStatusState")]
    pub enum CheckStatusState {
        Completed,
        InProgress,
        Queued,
        Requested,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Status")]
    pub struct Status {
        pub state: StatusState,
        pub contexts: Vec<StatusContext>,
    }

    #[derive(cynic::Enum, Clone, Copy, Debug, PartialEq)]
    #[cynic(graphql_type = "StatusState")]
    pub enum StatusState {
        Error,
        Expected,
        Failure,
        Pending,
        Success,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "StatusContext")]
    pub struct StatusContext {
        pub created_at: DateTime,
    }

    #[derive(cynic::Enum, Clone, Copy, Debug, PartialEq)]
    #[cynic(graphql_type = "PullRequestState")]
    pub enum PullRequestState {
        Closed,
        Merged,
        Open,
    }

    #[derive(cynic::Scalar, Debug, Clone)]
    pub struct DateTime(pub chrono::DateTime<chrono::Utc>);
}

#[cynic::schema_for_derives(file = r#"src/github.schema.graphql"#)]
mod team_query {
    use super::{schema, User};

    #[derive(cynic::FragmentArguments, Debug)]
    pub struct TeamMembersArguments {
        pub org: String,
        pub team: String,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Query", argument_struct = "TeamMembersArguments")]
    pub struct TeamMembers {
        #[arguments(login = args.org.clone())]
        pub organization: Option<Organization>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(
        graphql_type = "Organization",
        argument_struct = "TeamMembersArguments"
    )]
    pub struct Organization {
        #[arguments(slug = args.team.clone())]
        pub team: Option<Team>,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "Team")]
    pub struct Team {
        #[arguments(first = 100)]
        pub members: TeamMemberConnection,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "TeamMemberConnection")]
    pub struct TeamMemberConnection {
        #[cynic(flatten)]
        pub nodes: Vec<User>,
    }
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "User", schema_path = r#"src/github.schema.graphql"#)]
pub struct User {
    pub login: String,
}

mod schema {
    cynic::use_schema!(r#"src/github.schema.graphql"#);
}
