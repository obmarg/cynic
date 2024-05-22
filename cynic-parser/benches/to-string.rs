use divan::AllocProfiler;

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    // Run registered benchmarks.
    divan::main();
}

mod executable {
    const QUERY: &str = include_str!("../tests/executables/kitchen-sink.graphql");

    #[divan::bench]
    fn cynic_parser(bencher: divan::Bencher) {
        bencher
            .with_inputs(|| cynic_parser::parse_executable_document(QUERY).unwrap())
            .bench_refs(|document| document.to_string_pretty());
    }

    #[divan::bench(
        types = [
            &str,
            String
        ]
    )]
    fn graphql_parser<T>(bencher: divan::Bencher)
    where
        T: graphql_parser::query::Text<'static>,
    {
        bencher
            .with_inputs(|| graphql_parser::parse_query::<T>(QUERY).unwrap())
            .bench_refs(|document| document.to_string());
    }
}

mod type_system {
    const GITHUB_SCHEMA: &str = include_str!("../../schemas/github.graphql");

    #[divan::bench]
    fn cynic_parser(bencher: divan::Bencher) {
        bencher
            .with_inputs(|| cynic_parser::parse_type_system_document(GITHUB_SCHEMA).unwrap())
            .bench_refs(|document| document.to_sdl_pretty());
    }

    #[divan::bench(
        types = [
            &str,
            String
        ]
    )]
    fn graphql_parser<T>(bencher: divan::Bencher)
    where
        T: graphql_parser::query::Text<'static>,
    {
        bencher
            .with_inputs(|| graphql_parser::parse_schema::<T>(GITHUB_SCHEMA).unwrap())
            .bench_refs(|document| document.to_string());
    }
}
