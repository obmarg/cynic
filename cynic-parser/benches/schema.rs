// use divan::AllocProfiler;

// #[global_allocator]
// static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    // Run registered benchmarks.
    divan::main();
}

const GITHUB_SCHEMA: &str = include_str!("../../schemas/github.graphql");

#[divan::bench]
fn cynic_parser() -> cynic_parser::TypeSystemDocument {
    let parsed = cynic_parser::parse_type_system_document(GITHUB_SCHEMA).unwrap();
    divan::black_box(parsed)
}

#[divan::bench(
    types = [
        &str,
        String
    ]
)]
fn graphql_parser<T>() -> graphql_parser::schema::Document<'static, T>
where
    T: graphql_parser::query::Text<'static>,
{
    let parsed = graphql_parser::parse_schema(GITHUB_SCHEMA).unwrap();
    divan::black_box(parsed)
}

#[divan::bench]
fn async_graphql_parser() -> async_graphql_parser::types::ServiceDocument {
    let parsed = async_graphql_parser::parse_schema(GITHUB_SCHEMA).unwrap();
    divan::black_box(parsed)
}

#[divan::bench]
fn apollo_parser() -> apollo_parser::SyntaxTree {
    let parser = apollo_parser::Parser::new(GITHUB_SCHEMA);
    let cst = parser.parse();
    divan::black_box(cst)
}
