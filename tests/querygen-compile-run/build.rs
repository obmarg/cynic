use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use cynic_querygen::{document_to_fragment_structs, QueryGenOptions};

fn main() {
    let starwars_schema = Schema::from_repo_schemas(
        "https://swapi-graphql.netlify.app/.netlify/functions/index",
        "starwars.schema.graphql",
    );
    let jobs_schema =
        Schema::from_repo_schemas("https://api.graphql.jobs/", "graphql.jobs.graphql");
    let github_schema =
        Schema::from_repo_schemas("https://api.github.com/graphql", "github.graphql");

    let cases = &[
        TestCase::query(
            &starwars_schema,
            "../../cynic-querygen/tests/queries/starwars/sanity.graphql",
            r#"queries::SanityCheckQuery::build(
                &queries::SanityCheckQueryArguments {
                    film_id: Some("ZmlsbXM6MQ==".into())
                }
            )"#,
        ),
        TestCase::query(
            &starwars_schema,
            "../../cynic-querygen/tests/queries/starwars/nested-arguments.graphql",
            r#"queries::NestedArgsQuery::build(
                &queries::NestedArgsQueryArguments {
                    film_id: "ZmlsbXM6MQ==".into(),
                    planet_cursor: None,
                    resident_connection: None
                }
            )"#,
        ),
        TestCase::query(
            &starwars_schema,
            "../../cynic-querygen/tests/queries/starwars/bare-selection-set.graphql",
            r#"queries::UnnamedQuery::build(())"#,
        ),
        TestCase::query(
            &starwars_schema,
            "../../cynic-querygen/tests/queries/starwars/multiple-queries.graphql",
            r#"queries::AllFilms::build(())"#,
        ),
        TestCase::query(
            &starwars_schema,
            "../../cynic-querygen/tests/queries/starwars/fragment-spreads.graphql",
            r#"queries::AllFilms::build(())"#,
        ),
        TestCase::query(
            &jobs_schema,
            "tests/queries/graphql.jobs/london-jobs.graphql",
            r#"queries::Jobs::build(())"#,
        ),
        TestCase::query(
            &jobs_schema,
            "tests/queries/graphql.jobs/jobs.graphql",
            r#"queries::Jobs::build(
                &queries::JobsArguments {
                    input: queries::LocationInput {
                        slug: "london".into()
                    }
                }
            )"#,
        ),
        TestCase::mutation(
            &github_schema,
            "../../cynic-querygen/tests/queries/github/add-comment-mutation.graphql",
            r#"queries::CommentOnMutationSupportIssue::build(
                queries::CommentOnMutationSupportIssueArguments {
                    comment_body: "This is a test comment, posted by the new cynic mutation support"
                        .into(),
                },
            )"#,
        ),
        TestCase::query_norun(
            &github_schema,
            "../../cynic-querygen/tests/queries/github/input-object-arguments.graphql",
            r#"queries::PullRequestTitles::build(
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
            "tests/queries/github/nested-arguments.graphql",
            r#"queries::PullRequestTitles::build(
                queries::PullRequestTitlesArguments {
                    owner: "obmarg".into(),
                    repo: "cynic".into(),
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
            r#"queries::PullRequestTitles::build(())"#,
        ),
        TestCase::mutation(
            &github_schema,
            "tests/queries/github/scalar-inside-input-object.graphql",
            r#"queries::AddPRComment::build(
                &queries::AddPRCommentArguments{
                    body: "hello!".into(),
                    commit: queries::GitObjectID("abcd".into())
                }
            )"#,
        ),
        TestCase::query_norun(
            &github_schema,
            "../../cynic-querygen/tests/queries/github/literal-enums.graphql",
            r#"queries::RepoIssues::build(())"#,
        ),
        TestCase::query_norun(
            &github_schema,
            "tests/queries/github/optional-input-object-argument.graphql",
            r#"queries::PullRequestTitles::build(
                queries::PullRequestTitlesArguments {
                    pr_order: None
                },
            )"#,
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

        write!(
            &mut file,
            r#"
            #![allow(unused_imports)]

            fn main() {{
                {norun_code}
                use cynic::{{QueryBuilder, MutationBuilder}};
                querygen_compile_run::send("{url}", {operation_construct}).unwrap();
            }}

            {query_code}
            "#,
            norun_code = norun_code,
            url = self.schema.query_url,
            query_code = query_code,
            operation_construct = self.operation_construct
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
    fn from_repo_schemas(query_url: impl Into<String>, path: impl Into<PathBuf>) -> Schema {
        let example_path = PathBuf::from("../../schemas/");
        let path = example_path.join(path.into());
        Schema {
            query_url: query_url.into(),
            path_for_loading: path.clone(),
            path_for_generated_code: PathBuf::from("./..").join(path),
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
