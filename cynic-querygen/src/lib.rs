use inflector::Inflector;
use std::rc::Rc;

mod naming;
mod output;
mod query_parsing;
mod schema;
mod type_ext;

use schema::{GraphPath, TypeIndex};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Query document not supported: {0}")]
    UnsupportedQueryDocument(String),

    #[error("could not parse query document: {0}")]
    QueryParseError(#[from] graphql_parser::query::ParseError),

    #[error("could not parse schema document: {0}")]
    SchemaParseError(#[from] graphql_parser::schema::ParseError),

    #[error("could not find field `{0}` on `{1}`")]
    UnknownField(String, String),

    #[error("could not find enum `{0}`")]
    UnknownEnum(String),

    #[error("could not find type `{0}`")]
    UnknownType(String),

    #[error("expected type `{0}` to be an object")]
    ExpectedObject(String),

    #[error("expected type `{0}` to be an input object")]
    ExpectedInputObject(String),

    #[error("found a literal object but the argument is not an InputObject")]
    ArgumentNotInputObject,

    #[error("couldn't find an argument named `{0}`")]
    UnknownArgument(String),

    #[error("an enum-like value was provided to an argument that is not an enum")]
    ArgumentNotEnum,

    #[error("expected an input object, enum or scalar")]
    ExpectedInputType,

    #[error("expected an enum, scalar, object, union or interface")]
    ExpectedOutputType,

    #[error("expected a homogenous list of input values")]
    ExpectedHomogenousList,

    #[error("expected field to be a list")]
    ExpectedListType,

    #[error("expected a value that is or contains an input object")]
    ExpectedInputObjectValue,

    #[error("couldn't find a fragment named {0}")]
    UnknownFragment(String),

    #[error("Tried to apply a fragment for a {0} type on a {1} type")]
    TypeConditionFailed(String, String),

    #[error("Could not find a type named {0}, which we expected to be the root type")]
    CouldntFindRootType(String),

    #[error("At least one field should be selected for `{0}`.")]
    NoFieldSelected(String),
}

#[derive(Debug)]
pub struct QueryGenOptions {
    pub schema_path: String,
    pub query_module: String,
}

impl Default for QueryGenOptions {
    fn default() -> QueryGenOptions {
        QueryGenOptions {
            schema_path: "schema.graphql".into(),
            query_module: "query_dsl".into(),
        }
    }
}

pub fn document_to_fragment_structs(
    query: impl AsRef<str>,
    schema: impl AsRef<str>,
    options: &QueryGenOptions,
) -> Result<String, Error> {
    use output::indented;
    use std::fmt::Write;

    let schema = graphql_parser::parse_schema::<&str>(schema.as_ref())?;
    let query = graphql_parser::parse_query::<&str>(query.as_ref())?;

    let type_index = Rc::new(TypeIndex::from_schema(&schema));
    let parsed_output = query_parsing::parse_query_document(&query, &type_index)?;

    let mut output = String::new();

    // https://github.com/obmarg/cynic/issues/71
    let scalar_count = schema
        .definitions
        .iter()
        .filter(|def| {
            matches!(
                def,
                schema::Definition::TypeDefinition(schema::TypeDefinition::Scalar(_))
            )
        })
        .count();

    writeln!(output, "#[cynic::query_module(").unwrap();
    writeln!(output, "    schema_path = r#\"{}\"#,", options.schema_path).unwrap();
    writeln!(output, "    query_module = \"{}\",", options.query_module).unwrap();
    writeln!(output, ")]\nmod queries {{").unwrap();
    if scalar_count > 0 {
        writeln!(
            output,
            "    use super::{{{}, types::*}};\n",
            options.query_module
        )
        .unwrap();
    } else {
        writeln!(output, "    use super::{};\n", options.query_module).unwrap();
    }

    for argument_struct in parsed_output.argument_structs {
        writeln!(indented(&mut output, 4), "{}", argument_struct).unwrap();
    }

    for fragment in parsed_output.query_fragments {
        writeln!(indented(&mut output, 4), "{}", fragment).unwrap();
    }

    for en in parsed_output.enums {
        writeln!(indented(&mut output, 4), "{}", en).unwrap();
    }

    for input_object in parsed_output.input_objects {
        writeln!(indented(&mut output, 4), "{}", input_object).unwrap();
    }
    writeln!(output, "}}\n").unwrap();

    if scalar_count > 0 {
        writeln!(output, "#[cynic::query_module(").unwrap();
        writeln!(output, "    schema_path = r#\"{}\"#,", options.schema_path).unwrap();
        writeln!(output, "    query_module = \"{}\",", options.query_module).unwrap();
        writeln!(output, ")]\nmod types {{").unwrap();

        // Output any custom scalars we need.
        // Note that currently our query_dsl needs _all_ scalars in a schema
        // so we're parsing this out from schema.definitons rather than output.scalars
        for def in &schema.definitions {
            if let schema::Definition::TypeDefinition(schema::TypeDefinition::Scalar(scalar)) = def
            {
                writeln!(output, "    #[derive(cynic::Scalar, Debug, Clone)]").unwrap();
                writeln!(
                    output,
                    "    pub struct {}(pub String);\n",
                    scalar.name.to_pascal_case()
                )
                .unwrap();
            }
        }
        writeln!(output, "}}\n").unwrap();
    }

    writeln!(output, "mod {}{{", options.query_module).unwrap();

    if scalar_count > 0 {
        writeln!(output, "    use super::types::*;").unwrap();
    }

    writeln!(
        output,
        "    cynic::query_dsl!(r#\"{}\"#);",
        options.schema_path
    )
    .unwrap();
    writeln!(output, "}}\n").unwrap();

    Ok(output)
}
