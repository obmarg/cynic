//! An example of querying the starwars API using the reqwest-blocking feature

mod query_dsl {
    cynic::query_dsl!("../schemas/graphql.jobs.graphql");
}

type DateTime = chrono::DateTime<chrono::Utc>;

cynic::impl_scalar!(DateTime, query_dsl::DateTime);

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "../schemas/graphql.jobs.graphql",
    query_module = "query_dsl",
    graphql_type = "Job"
)]
struct Job {
    created_at: DateTime,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "../schemas/graphql.jobs.graphql",
    query_module = "query_dsl",
    graphql_type = "Query"
)]
struct JobsQuery {
    jobs: Vec<Job>,
}

fn main() {
    let result = run_query();
    println!("{:?}", result);
}

fn run_query() -> cynic::GraphQLResponse<JobsQuery> {
    use cynic::http::ReqwestBlockingExt;

    let query = build_query();

    reqwest::blocking::Client::new()
        .post("https://api.graphql.jobs")
        .run_graphql(query)
        .unwrap()
}

fn build_query() -> cynic::Operation<'static, JobsQuery> {
    use cynic::QueryBuilder;

    JobsQuery::build(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn snapshot_test_menu_query() {
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
        insta::assert_debug_snapshot!(result.data);
    }
}
