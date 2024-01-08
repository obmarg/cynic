fn main() {
    // Run registered benchmarks.
    divan::main();
}

const GITHUB_SCHEMA: &str = include_str!("../../schemas/github.graphql");

#[divan::bench]
fn cynic_parser() {
    let parsed = cynic_parser::parse_type_system_document(GITHUB_SCHEMA);
    divan::black_box(parsed);
}

#[divan::bench]
fn graphql_parser_string() {
    let parsed = graphql_parser::parse_schema::<String>(GITHUB_SCHEMA).unwrap();
    divan::black_box(parsed);
}

#[divan::bench]
fn graphql_parser_str() {
    let parsed = graphql_parser::parse_schema::<&str>(GITHUB_SCHEMA).unwrap();
    divan::black_box(parsed);
}

#[divan::bench]
fn async_graphql_parser() {
    let parsed = async_graphql_parser::parse_schema(GITHUB_SCHEMA).unwrap();
    divan::black_box(parsed);
}

#[divan::bench]
fn apollo_parser() {
    let parser = apollo_parser::Parser::new(GITHUB_SCHEMA);
    let cst = parser.parse();
    divan::black_box(cst);
}
