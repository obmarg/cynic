use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use cynic_querygen::{document_to_fragment_structs, QueryGenOptions};

fn main() {
    let starwars_schema = SchemaPath::from_examples("starwars.schema.graphql");
    let jobs_schema = SchemaPath::from_test_schemas("graphql.jobs.gql");

    let cases = &[
        TestCase::new(
            &starwars_schema,
            "../../cynic-querygen/tests/queries/starwars/sanity.graphql",
            r#"queries::SanityCheckQuery::fragment(
                queries::SanityCheckQueryArguments {
                    film_id: Some("ZmlsbXM6MQ==".into())
                }
            )"#,
        ),
        TestCase::new(
            &starwars_schema,
            "../../cynic-querygen/tests/queries/starwars/nested-arguments.graphql",
            r#"queries::NestedArgsQuery::fragment(
                queries::NestedArgsQueryArguments {
                    film_id: "ZmlsbXM6MQ==".into(),
                    planet_cursor: None,
                    resident_connection: None
                }
            )"#,
        ),
        TestCase::new(
            &starwars_schema,
            "../../cynic-querygen/tests/queries/starwars/bare-selection-set.graphql",
            r#"queries::UnnamedQuery::fragment(())"#,
        ),
        TestCase::new(
            &jobs_schema,
            "tests/queries/graphql.jobs/london-jobs.graphql",
            r#"queries::Jobs::fragment(())"#,
        ),
        TestCase::new(
            &jobs_schema,
            "tests/queries/graphql.jobs/jobs.graphql",
            r#"queries::Jobs::fragment(
                queries::JobsArguments {
                    input: Some(queries::LocationInput {
                        slug: "london".into()
                    })
                }
            )"#,
        ),
    ];

    for case in cases {
        case.write_code();
    }
}

struct TestCase {
    schema_path: SchemaPath,
    query_path: PathBuf,
    fragment_construct: String,
}

impl TestCase {
    fn new(
        schema_path: &SchemaPath,
        query_path: impl Into<PathBuf>,
        fragment_construct: impl Into<String>,
    ) -> Self {
        TestCase {
            query_path: query_path.into(),
            schema_path: schema_path.clone(),
            fragment_construct: fragment_construct.into(),
        }
    }

    fn write_code(&self) {
        let schema = load_file(&self.schema_path.for_loading);
        let query = load_file(&self.query_path);

        let options = QueryGenOptions {
            schema_path: self.schema_path.for_generated_code.to_str().unwrap().into(),
            ..QueryGenOptions::default()
        };
        let query_code = document_to_fragment_structs(&query, &schema, &options).unwrap();

        // TODO: Now need to output query_code inside some sort of viable template.rs file.
        // Could use proc_macro & quote to do this _if_ I ran cargo fmt after...
        //
        // Ideally I have a func that does the build-a-query & run it through HTTP stuff
        // somewhere that the examples can pull in.
        //
        // Then I just need to output fragment_structs & a main function that ties the two
        // together...

        let test_filename = {
            let mut path = self.query_path.clone();
            path.set_extension("rs");
            path.file_name().unwrap().to_str().unwrap().to_owned()
        };

        let mut file = File::create(format!("tests/generated/{}", test_filename)).unwrap();

        // TODO: So, need a way to know the query struct name here.
        // Also need to be able to construct any variable structs...
        write!(
            &mut file,
            r#"
            #![allow(unused_imports)]

            fn main() {{
                use cynic::QueryFragment;
                querygen_compile_run::send_query({fragment_construct}).unwrap();
            }}

            {query_code}
            "#,
            query_code = query_code,
            fragment_construct = self.fragment_construct
        )
        .unwrap();
    }
}

#[derive(Clone, Debug)]
struct SchemaPath {
    for_generated_code: PathBuf,
    for_loading: PathBuf,
}

impl SchemaPath {
    /// Constructs a SchemaPath from the examples package
    fn from_examples(path: impl Into<PathBuf>) -> SchemaPath {
        let example_path = PathBuf::from("../../examples/examples/");
        let path = example_path.join(path.into());
        SchemaPath {
            for_loading: path.clone(),
            for_generated_code: PathBuf::from("./..").join(path),
        }
    }

    /// Constructs a SchemaPath from the examples package
    fn from_test_schemas(path: impl Into<PathBuf>) -> SchemaPath {
        let test_schema_path = PathBuf::from("tests/schemas");
        let path = test_schema_path.join(path.into());
        SchemaPath {
            for_loading: path.clone(),
            for_generated_code: PathBuf::from("./../../../tests/querygen-compile-run").join(path),
        }
    }
}

fn load_file(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();
    println!("cargo:rerun-if-changed={}", path.to_str().unwrap());

    let mut contents = String::new();
    File::open(path)
        .unwrap()
        .read_to_string(&mut contents)
        .unwrap();

    contents
}
