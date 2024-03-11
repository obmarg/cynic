use divan::AllocProfiler;

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    // Run registered benchmarks.
    divan::main();
}

const QUERY: &str = include_str!("../tests/executables/kitchen-sink.graphql");

#[divan::bench]
fn cynic_parser() -> cynic_parser::ExecutableDocument {
    let parsed = cynic_parser::parse_executable_document(QUERY).unwrap();
    divan::black_box(parsed)
}

#[divan::bench(
    types = [
        &str,
        String
    ]
)]
fn graphql_parser<T>() -> graphql_parser::query::Document<'static, T>
where
    T: graphql_parser::query::Text<'static>,
{
    let parsed = graphql_parser::parse_query(QUERY).unwrap();
    divan::black_box(parsed)
}

#[divan::bench]
fn async_graphql_parser() -> async_graphql_parser::types::ExecutableDocument {
    let parsed = async_graphql_parser::parse_query(QUERY).unwrap();
    divan::black_box(parsed)
}

#[divan::bench]
fn apollo_parser() -> apollo_parser::SyntaxTree {
    let parser = apollo_parser::Parser::new(QUERY);
    let cst = parser.parse();
    divan::black_box(cst)
}
