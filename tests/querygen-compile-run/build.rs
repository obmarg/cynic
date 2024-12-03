use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use indoc::writedoc;

use cynic_querygen::{document_to_fragment_structs, QueryGenOptions};

fn main() {
    let starwars_schema = Schema::from_repo_schemas(
        "starwars",
        "https://swapi-graphql.netlify.app/.netlify/functions/index",
        "starwars.schema.graphql",
    );
    let jobs_schema = Schema::from_repo_schemas(
        "graphql.jobs",
        "https://api.graphql.jobs/",
        "graphql.jobs.graphql",
    );
    let github_schema =
        Schema::from_repo_schemas("github", "https://api.github.com/graphql", "github.graphql");
    let book_schema = Schema::from_repo_schemas("book", "https://example.com", "books.graphql");

    let raindancer_schema =
        Schema::from_repo_schemas("raindancer", "https://example.com", "raindancer.graphql");

    let test_schema =
        Schema::from_repo_schemas("test-schema", "https://example.com", "test_cases.graphql");

    let cases = &[
        TestCase::query(
            &starwars_schema,
            "../../cynic-querygen/tests/queries/starwars/sanity.graphql",
            r#"SanityCheckQuery::build(
                SanityCheckQueryVariables {
                    film_id: Some((&String::from("ZmlsbXM6MQ==")).into())
                }
            )"#,
        ),
        TestCase::query(
            &starwars_schema,
            "../../cynic-querygen/tests/queries/starwars/nested-arguments.graphql",
            r#"NestedArgsQuery::build(
                NestedArgsQueryVariables {
                    film_id: &"ZmlsbXM6MQ==".into(),
                    planet_cursor: None,
                    resident_connection: None
                }
            )"#,
        ),
        TestCase::query(
            &starwars_schema,
            "../../cynic-querygen/tests/queries/starwars/bare-selection-set.graphql",
            r#"UnnamedQuery::build(())"#,
        ),
        TestCase::query(
            &starwars_schema,
            "../../cynic-querygen/tests/queries/starwars/multiple-queries.graphql",
            r#"AllFilms::build(())"#,
        ),
        TestCase::query(
            &starwars_schema,
            "../../cynic-querygen/tests/queries/starwars/fragment-spreads.graphql",
            r#"AllFilms::build(())"#,
        ),
        TestCase::query_norun(
            &jobs_schema,
            "tests/queries/graphql.jobs/london-jobs.graphql",
            r#"Jobs::build(())"#,
        ),
        TestCase::query_norun(
            &jobs_schema,
            "tests/queries/graphql.jobs/jobs.graphql",
            r#"Jobs::build(
                JobsVariables {
                    input: LocationInput {
                        slug: "london"
                    }
                }
            )"#,
        ),
        TestCase::mutation(
            &github_schema,
            "../../cynic-querygen/tests/queries/github/add-comment-mutation.graphql",
            r#"CommentOnMutationSupportIssue::build(
                CommentOnMutationSupportIssueVariables {
                    comment_body: "This is a test comment, posted by the new cynic mutation support"
                },
            )"#,
        ),
        TestCase::query_norun(
            &github_schema,
            "../../cynic-querygen/tests/queries/github/input-object-arguments.graphql",
            r#"PullRequestTitles::build(
                PullRequestTitlesVariables {
                    pr_order: IssueOrder {
                        direction: OrderDirection::Asc,
                        field: IssueOrderField::CreatedAt,
                    }
                },
            )"#,
        ),
        TestCase::query_norun(
            &github_schema,
            "tests/queries/github/nested-arguments.graphql",
            r#"PullRequestTitles::build(
                PullRequestTitlesVariables {
                    owner: "obmarg",
                    repo: "cynic",
                    pr_order: IssueOrder {
                        direction: OrderDirection::Asc,
                        field: IssueOrderField::CreatedAt,
                    }
                },
            )"#,
        ),
        TestCase::query_norun(
            &github_schema,
            "../../cynic-querygen/tests/queries/github/input-object-literals.graphql",
            r#"PullRequestTitles::build(())"#,
        ),
        TestCase::query_norun(
            &test_schema,
            "../../cynic-querygen/tests/queries/misc/scalar-casing.graphql",
            r#"MyQuery::build(
                MyQueryVariables {
                    id: Uuid("not-really-a-uuid-but-whatever".into())
                }
            )"#,
        ),
        TestCase::query_norun(
            &github_schema,
            "../../cynic-querygen/tests/queries/github/queries-with-typename.graphql",
            r#"UnnamedQuery::build(())"#,
        ),
        TestCase::query_norun(
            &github_schema,
            "../../cynic-querygen/tests/queries/github/issue-786.graphql",
            r#"ProjectMetadataQuery::build(
                ProjectMetadataQueryVariables {
                    id: &cynic::Id::new("123"),
                }
            )"#,
        ),
        TestCase::mutation(
            &github_schema,
            "tests/queries/github/scalar-inside-input-object.graphql",
            r#"AddPRComment::build(
                AddPRCommentVariables {
                    body: "hello!",
                    commit: GitObjectId("abcd".into())
                }
            )"#,
        ),
        TestCase::query_norun(
            &github_schema,
            "../../cynic-querygen/tests/queries/github/literal-enums.graphql",
            r#"RepoIssues::build(())"#,
        ),
        TestCase::query_norun(
            &github_schema,
            "tests/queries/github/optional-input-object-argument.graphql",
            r#"PullRequestTitles::build(
                PullRequestTitlesVariables {
                    pr_order: None
                },
            )"#,
        ),
        TestCase::mutation(
            &raindancer_schema,
            "tests/queries/misc/mutation_with_scalar_result_and_input.graphql",
            r#"SignIn::build(
                SignInVariables {
                    username: "hello",
                    password: "hello"
                },
            )"#,
        ),
        TestCase::query_norun(
            &github_schema,
            "tests/queries/github/inline-fragment-on-union.graphql",
            r#"IssueOrPR::build(())"#,
        ),
        TestCase::query_norun(
            &github_schema,
            "tests/queries/github/inline-fragment-with-arguments.graphql",
            r#"IssueOrPR::build(
                IssueOrPRVariables {
                    assignee_count: 10
                }
            )"#,
        ),
        TestCase::query_norun(
            &github_schema,
            "tests/queries/github/inline-fragment-with-renames.graphql",
            r#"RepoIssues::build(
                RepoIssuesVariables {
                    first: 10
                }
            )"#,
        ),
        TestCase::subscription(
            &book_schema,
            "tests/queries/books/books.graphql",
            r#"BookSubscription::build(())"#,
        ),
        TestCase::query_norun(
            &test_schema,
            "tests/queries/misc/keyword-argument.graphql",
            r#"KeywordArgumentQuery::build(())"#,
        ),
        TestCase::query_norun(
            &test_schema,
            "tests/queries/misc/recursive-inputs.graphql",
            r#"RecursiveInputQuery::build(RecursiveInputQueryVariables {
                input: None,
                input2: None
            })"#,
        ),
    ];

    for case in cases {
        case.write_code();
    }
}

struct TestCase {
    schema: Schema,
    query_path: PathBuf,
    operation_construct: String,
    should_run: bool,
    is_subscription: bool,
}

impl TestCase {
    fn query(
        schema: &Schema,
        query_path: impl Into<PathBuf>,
        operation_construct: impl Into<String>,
    ) -> Self {
        TestCase {
            query_path: query_path.into(),
            schema: schema.clone(),
            operation_construct: operation_construct.into(),
            should_run: true,
            is_subscription: false,
        }
    }

    fn query_norun(
        schema: &Schema,
        query_path: impl Into<PathBuf>,
        operation_construct: impl Into<String>,
    ) -> Self {
        TestCase {
            query_path: query_path.into(),
            schema: schema.clone(),
            operation_construct: operation_construct.into(),
            should_run: false,
            is_subscription: false,
        }
    }

    fn mutation(
        schema: &Schema,
        query_path: impl Into<PathBuf>,
        operation_construct: impl Into<String>,
    ) -> Self {
        TestCase {
            query_path: query_path.into(),
            schema: schema.clone(),
            operation_construct: operation_construct.into(),
            // We don't run mutations by default
            should_run: false,
            is_subscription: false,
        }
    }

    fn subscription(
        schema: &Schema,
        query_path: impl Into<PathBuf>,
        operation_construct: impl Into<String>,
    ) -> Self {
        TestCase {
            query_path: query_path.into(),
            schema: schema.clone(),
            operation_construct: operation_construct.into(),
            // We don't run subscriptions by default
            should_run: false,
            is_subscription: true,
        }
    }

    fn write_code(&self) {
        let schema = load_file(&self.schema.path_for_loading);
        let query = load_file(&self.query_path);

        let options = QueryGenOptions {
            schema_name: Some(self.schema.schema_name.clone()),
            ..QueryGenOptions::default()
        };
        let query_code = document_to_fragment_structs(query, schema, &options).unwrap();

        let test_filename = {
            let mut path = self.query_path.clone();
            path.set_extension("rs");
            path.file_name().unwrap().to_str().unwrap().to_owned()
        };

        let mut file = File::create(format!("tests/{}", test_filename)).unwrap();

        let norun_code = if self.should_run {
            ""
        } else {
            "#![allow(unreachable_code)] return;"
        };

        let run_code = if self.is_subscription {
            format!("{};", &self.operation_construct)
        } else {
            format!(
                r#"querygen_compile_run::send("{url}", {operation_construct}).unwrap();"#,
                url = self.schema.query_url,
                operation_construct = self.operation_construct
            )
        };

        writedoc!(
            &mut file,
            r#"
            #![allow(unused_imports, clippy::large_enum_variant)]

            #[test]
            fn generated_test() {{
                {norun_code}
                use cynic::{{QueryBuilder, MutationBuilder, SubscriptionBuilder}};
                {run_code}
            }}

            {query_code}

            #[cynic::schema("{schema_name}")]
            mod schema {{}}
            "#,
            norun_code = norun_code,
            query_code = query_code,
            run_code = run_code,
            schema_name = self.schema.schema_name
        )
        .unwrap();
    }
}

#[derive(Clone, Debug)]
struct Schema {
    query_url: String,
    schema_name: String,
    path_for_loading: PathBuf,
}

impl Schema {
    /// Constructs a SchemaPath from the examples package
    fn from_repo_schemas(
        schema_name: impl Into<String>,
        query_url: impl Into<String>,
        path: impl Into<PathBuf>,
    ) -> Schema {
        let schema_dir = PathBuf::from("../../schemas/");
        let path = schema_dir.join(path.into());
        let schema_name = schema_name.into();

        cynic_codegen::register_schema(&schema_name)
            .from_sdl_file(&path)
            .unwrap();

        Schema {
            query_url: query_url.into(),
            path_for_loading: path,
            schema_name,
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
