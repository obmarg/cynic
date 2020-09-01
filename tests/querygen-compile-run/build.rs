use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use cynic_querygen::{document_to_fragment_structs, QueryGenOptions};

fn main() {
    let starwars_schema = Schema::from_examples(
        "https://swapi-graphql.netlify.com/.netlify/functions/index",
        "starwars.schema.graphql",
    );
    let jobs_schema = Schema::from_test_schemas("https://api.graphql.jobs/", "graphql.jobs.gql");
    let github_schema =
        Schema::from_querygen_tests("https://api.github.com/graphql", "github.graphql");

    let cases = &[
        TestCase::query(
            &starwars_schema,
            "../../cynic-querygen/tests/queries/starwars/sanity.graphql",
            r#"queries::SanityCheckQuery::fragment(
                queries::SanityCheckQueryArguments {
                    film_id: Some("ZmlsbXM6MQ==".into())
                }
            )"#,
        ),
        TestCase::query(
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
        TestCase::query(
            &starwars_schema,
            "../../cynic-querygen/tests/queries/starwars/bare-selection-set.graphql",
            r#"queries::UnnamedQuery::fragment(())"#,
        ),
        TestCase::query(
            &jobs_schema,
            "tests/queries/graphql.jobs/london-jobs.graphql",
            r#"queries::Jobs::fragment(())"#,
        ),
        TestCase::query(
            &jobs_schema,
            "tests/queries/graphql.jobs/jobs.graphql",
            r#"queries::Jobs::fragment(
                queries::JobsArguments {
                    input: queries::LocationInput {
                        slug: "london".into()
                    }
                }
            )"#,
        ),
        TestCase::mutation(
            &github_schema,
            "../../cynic-querygen/tests/queries/github/add-comment-mutation.graphql",
            r#"queries::CommentOnMutationSupportIssue::fragment(
                queries::CommentOnMutationSupportIssueArguments {
                    comment_body: "This is a test comment, posted by the new cynic mutation support"
                        .into(),
                },
            )"#,
        ),
        TestCase::query_norun(
            &github_schema,
            "../../cynic-querygen/tests/queries/github/input-object-arguments.graphql",
            r#"queries::PullRequestTitles::fragment(
                queries::PullRequestTitlesArguments {
                    pr_order: queries::IssueOrder {
                        direction: queries::OrderDirection::Asc,
                        field: queries::IssueOrderField::CreatedAt,
                    }
                },
            )"#,
        ),
        TestCase::query_norun(
            &github_schema,
            "../../cynic-querygen/tests/queries/github/input-object-literals.graphql",
            r#"queries::PullRequestTitles::fragment(())"#,
        ),
        TestCase::query_norun(
            &github_schema,
            "../../cynic-querygen/tests/queries/github/literal-enums.graphql",
            r#"queries::Query::fragment(())"#,
        ),
    ];

    for case in cases {
        case.write_code();
    }
}

struct TestCase {
    schema: Schema,
    query_path: PathBuf,
    fragment_construct: String,
    should_run: bool,
    mutation: bool,
}

impl TestCase {
    fn query(
        schema: &Schema,
        query_path: impl Into<PathBuf>,
        fragment_construct: impl Into<String>,
    ) -> Self {
        TestCase {
            query_path: query_path.into(),
            schema: schema.clone(),
            fragment_construct: fragment_construct.into(),
            should_run: true,
            mutation: false,
        }
    }

    fn query_norun(
        schema: &Schema,
        query_path: impl Into<PathBuf>,
        fragment_construct: impl Into<String>,
    ) -> Self {
        TestCase {
            query_path: query_path.into(),
            schema: schema.clone(),
            fragment_construct: fragment_construct.into(),
            should_run: false,
            mutation: false,
        }
    }

    fn mutation(
        schema: &Schema,
        query_path: impl Into<PathBuf>,
        fragment_construct: impl Into<String>,
    ) -> Self {
        TestCase {
            query_path: query_path.into(),
            schema: schema.clone(),
            fragment_construct: fragment_construct.into(),
            // We don't run mutations by default
            should_run: false,
            mutation: true,
        }
    }

    fn write_code(&self) {
        let schema = load_file(&self.schema.path_for_loading);
        let query = load_file(&self.query_path);

        let options = QueryGenOptions {
            schema_path: self.schema.path_for_generated_code.to_str().unwrap().into(),
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

        let norun_code = if self.should_run {
            ""
        } else {
            "#![allow(unreachable_code)] return;"
        };

        let operation_function = if self.mutation {
            "send_mutation"
        } else {
            "send_query"
        };

        // TODO: So, need a way to know the query struct name here.
        // Also need to be able to construct any variable structs...
        write!(
            &mut file,
            r#"
            #![allow(unused_imports)]

            fn main() {{
                {norun_code}
                use cynic::QueryFragment;
                querygen_compile_run::{operation_function}("{url}", {fragment_construct}).unwrap();
            }}

            {query_code}
            "#,
            norun_code = norun_code,
            operation_function = operation_function,
            url = self.schema.query_url,
            query_code = query_code,
            fragment_construct = self.fragment_construct
        )
        .unwrap();
    }
}

#[derive(Clone, Debug)]
struct Schema {
    query_url: String,
    path_for_generated_code: PathBuf,
    path_for_loading: PathBuf,
}

impl Schema {
    /// Constructs a SchemaPath from the examples package
    fn from_examples(query_url: impl Into<String>, path: impl Into<PathBuf>) -> Schema {
        let example_path = PathBuf::from("../../examples/examples/");
        let path = example_path.join(path.into());
        Schema {
            query_url: query_url.into(),
            path_for_loading: path.clone(),
            path_for_generated_code: PathBuf::from("./..").join(path),
        }
    }

    /// Constructs a SchemaPath from this package
    fn from_test_schemas(query_url: impl Into<String>, path: impl Into<PathBuf>) -> Schema {
        let test_schema_path = PathBuf::from("tests/schemas");
        let path = test_schema_path.join(path.into());
        Schema {
            query_url: query_url.into(),
            path_for_loading: path.clone(),
            path_for_generated_code: PathBuf::from("./../../../tests/querygen-compile-run")
                .join(path),
        }
    }

    /// Constructs a SchemaPath from this package
    fn from_querygen_tests(query_url: impl Into<String>, path: impl Into<PathBuf>) -> Schema {
        let test_schema_path = PathBuf::from("../../cynic-querygen/tests/schemas");
        let path = test_schema_path.join(path.into());
        Schema {
            query_url: query_url.into(),
            path_for_loading: path.clone(),
            path_for_generated_code: PathBuf::from("./../").join(path),
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
