macro_rules! schema_tests {
    ($name: ident, $path: literal) => {
        mod $name {
            use similar_asserts::assert_eq;

            #[allow(non_upper_case_globals)]
            const SCHEMA: &str = include_str!($path);

            #[test]
            fn snapshot() {
                let parsed = cynic_parser::parse_type_system_document(SCHEMA)
                    .map_err(|error| error.to_report(SCHEMA))
                    .unwrap();
                insta::assert_snapshot!(parsed.to_sdl_pretty());
            }

            #[test]
            fn double_roundtrip() {
                let parsed = cynic_parser::parse_type_system_document(SCHEMA)
                    .map_err(|error| error.to_report(SCHEMA))
                    .unwrap();
                let sdl = parsed.to_sdl_pretty();

                let reparsed = cynic_parser::parse_type_system_document(&sdl)
                    .map_err(|error| error.to_report(&sdl))
                    .unwrap();

                assert_eq!(sdl, reparsed.to_sdl_pretty());
            }
        }
    };
}

schema_tests!(github, "../../schemas/github.graphql");

schema_tests!(books, "../../schemas/books.graphql");

schema_tests!(graphql_jobs, "../../schemas/graphql.jobs.graphql");

schema_tests!(raindancer, "../../schemas/raindancer.graphql");

schema_tests!(simple, "../../schemas/simple.graphql");

schema_tests!(starwars, "../../schemas/starwars.schema.graphql");

schema_tests!(test_cases, "../../schemas/test_cases.graphql");
