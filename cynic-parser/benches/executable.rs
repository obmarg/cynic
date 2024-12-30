// use divan::AllocProfiler;
// #[global_allocator]
// static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    // Run registered benchmarks.
    divan::main();
}

mod kitchen_sink {
    const QUERY: &str = include_str!("../tests/executables/kitchen-sink.graphql");

    #[divan::bench]
    fn cynic_parser() -> cynic_parser::ExecutableDocument {
        cynic_parser::parse_executable_document(QUERY).unwrap()
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
        graphql_parser::parse_query(QUERY).unwrap()
    }

    #[divan::bench]
    fn async_graphql_parser() -> async_graphql_parser::types::ExecutableDocument {
        async_graphql_parser::parse_query(QUERY).unwrap()
    }

    #[divan::bench]
    fn apollo_parser() -> apollo_parser::SyntaxTree {
        apollo_parser::Parser::new(QUERY).parse()
    }
}

mod introspection {
    const QUERY: &str = include_str!("../tests/executables/introspection.graphql");

    #[divan::bench]
    fn cynic_parser() -> cynic_parser::ExecutableDocument {
        cynic_parser::parse_executable_document(QUERY).unwrap()
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
        graphql_parser::parse_query(QUERY).unwrap()
    }

    #[divan::bench]
    fn async_graphql_parser() -> async_graphql_parser::types::ExecutableDocument {
        async_graphql_parser::parse_query(QUERY).unwrap()
    }

    #[divan::bench]
    fn apollo_parser() -> apollo_parser::SyntaxTree {
        apollo_parser::Parser::new(QUERY).parse()
    }
}
