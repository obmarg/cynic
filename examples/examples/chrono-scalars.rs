//! An example of querying the starwars API using the reqwest-blocking feature

mod schema {
    // We didn't register the graphql.jobs schema in our build.rs
    // so we bring it in here with `use_schema!`
    cynic::use_schema!("../schemas/graphql.jobs.graphql");
}

type DateTime = chrono::DateTime<chrono::Utc>;

cynic::impl_scalar!(DateTime, schema::DateTime);

#[derive(cynic::QueryFragment, Debug)]
#[cynic(schema_path = "../schemas/graphql.jobs.graphql")]
struct Job {
    created_at: DateTime,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    schema_path = "../schemas/graphql.jobs.graphql",
    graphql_type = "Query"
)]
struct JobsQuery {
    jobs: Vec<Job>,
}

fn main() {
    let result = run_query();
    for job in result.data.unwrap().jobs {
        println!("{}", job.created_at);
    }
}

fn run_query() -> cynic::GraphQlResponse<JobsQuery> {
    use cynic::http::ReqwestBlockingExt;

    let query = build_query();

    reqwest::blocking::Client::new()
        .post("https://api.graphql.jobs")
        .run_graphql(&query)
        .unwrap()
}

fn build_query() -> cynic::Operation<JobsQuery, ()> {
    use cynic::QueryBuilder;

    JobsQuery::build(())
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

    // Disabling this for now as the graphql jobs API doesn't actually work anymore :(
    #[cfg(nope)]
    #[test]
    fn test_running_query() {
        let result = run_query();
        if result.errors.is_some() {
            assert_eq!(result.errors.unwrap().len(), 0);
        }
        insta::assert_debug_snapshot!(result.data);
    }
}
