/// Safely run queries using generated query objects, 
/// which have been checked against the underlying schema.
/// We do not need to codegen again.
use query::*;

fn main() {
    let result = run_query();
    println!("{:#?}", result);
}

fn run_query() -> cynic::GraphQlResponse<PullRequestTitles> {
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

fn build_query() -> cynic::Operation<PullRequestTitles, PullRequestTitlesArguments> {
    use cynic::QueryBuilder;

    PullRequestTitles::build(PullRequestTitlesArguments {
        pr_order: IssueOrder {
            direction: OrderDirection::Asc,
            field: IssueOrderField::CreatedAt,
        },
    })
}