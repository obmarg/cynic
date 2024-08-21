//! An example of inlining variables into a GraphQL operation prior to sending
//!
//! This example uses the starwars API but the use case is primarily to support
//! shopifies [bulkOperationRunQuery][1] which requires a document with no variables.
//!
//! [1]: https://shopify.dev/docs/api/admin-graphql/2024-07/mutations/bulkoperationrunquery

use cynic::queries::{build_executable_document, OperationType};
use graphql_query::visit::FoldNode;

// Pull in the Star Wars schema we registered in build.rs
#[cynic::schema("starwars")]
mod schema {}

#[derive(cynic::QueryFragment, Debug)]
struct Film {
    title: Option<String>,
    director: Option<String>,
}

#[derive(cynic::QueryVariables)]
struct InlineVariables {
    id: Option<cynic::Id>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Root", variables = "InlineVariables")]
struct FilmDirectorQuery {
    #[arguments(id: $id)]
    film: Option<Film>,
}

fn main() {
    match run_query().data {
        Some(FilmDirectorQuery { film: Some(film) }) => {
            println!("{:?} was directed by {:?}", film.title, film.director)
        }
        _ => {
            println!("No film found");
        }
    }
}

fn run_query() -> cynic::GraphQlResponse<FilmDirectorQuery> {
    use cynic::http::ReqwestBlockingExt;

    let query = build_query();

    reqwest::blocking::Client::new()
        .post("https://swapi-graphql.netlify.app/.netlify/functions/index")
        .run_graphql(query)
        .unwrap()
}

fn build_query() -> cynic::Operation<FilmDirectorQuery, ()> {
    let document = build_executable_document::<FilmDirectorQuery, InlineVariables>(
        OperationType::Query,
        None,
        Default::default(),
    );

    let variables = InlineVariables {
        id: Some("ZmlsbXM6MQ==".into()),
    };

    use graphql_query::ast::*;

    let ctx = ASTContext::new();
    let ast = Document::parse(&ctx, document).unwrap();

    let query = ast
        .fold(&ctx, &mut VariableInline { variables })
        .unwrap()
        .print();

    cynic::Operation::new(query, ())
}

struct VariableInline {
    variables: InlineVariables,
}

impl<'a> graphql_query::visit::Folder<'a> for VariableInline {
    fn value(
        &mut self,
        ctx: &'a graphql_query::ast::ASTContext,
        value: graphql_query::ast::Value<'a>,
        _info: &graphql_query::visit::VisitInfo,
    ) -> graphql_query::visit::Result<graphql_query::ast::Value<'a>> {
        use graphql_query::ast::{StringValue, Value};

        let Value::Variable(variable) = value else {
            return Ok(value);
        };

        Ok(match variable.name {
            "id" => self
                .variables
                .id
                .as_ref()
                .map(|id| Value::String(StringValue::new(ctx, id.inner())))
                .unwrap_or(Value::Null),
            _ => Value::Null,
        })
    }

    fn variable_definitions(
        &mut self,
        ctx: &'a graphql_query::ast::ASTContext,
        _var_defs: graphql_query::ast::VariableDefinitions<'a>,
        _info: &graphql_query::visit::VisitInfo,
    ) -> graphql_query::visit::Result<graphql_query::ast::VariableDefinitions<'a>> {
        Ok(graphql_query::ast::VariableDefinitions {
            children: graphql_query::bumpalo::vec![in &ctx.arena],
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn snapshot_test_query() {
        // Running a snapshot test of the query building functionality as that gives us
        // a place to copy and paste the actual GQL we're using for running elsewhere,
        // and also helps ensure we don't change queries by mistake

        let query = build_query();

        insta::assert_snapshot!(query.query);
    }

    #[test]
    fn test_running_query() {
        let result = run_query();
        if result.errors.is_some() {
            assert_eq!(result.errors.unwrap().len(), 0);
        }
        insta::assert_debug_snapshot!(result.data);
    }
}
