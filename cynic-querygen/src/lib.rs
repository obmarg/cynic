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

    #[error("expected an interface")]
    ExpectedInterfaceType,

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

    #[error("Tried to apply an inline fragment on the {0} type which is not a union or interface")]
    InlineFragmentOnUnsupportedType(String),

    #[error("{0} is not a member of the {1} union type")]
    TypeNotUnionMember(String, String),

    #[error("{0} does not implement the {1} interface")]
    TypeDoesNotImplementInterface(String, String),

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
            query_module: "schema".into(),
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

    writeln!(output, "#[cynic::schema_for_derives(").unwrap();
    writeln!(output, "    file = r#\"{}\"#,", options.schema_path).unwrap();
    writeln!(output, "    module = \"{}\",", options.query_module).unwrap();
    writeln!(output, ")]\nmod queries {{").unwrap();

    let mod_output = &mut indented(&mut output, 4);

    writeln!(mod_output, "use super::{};\n", options.query_module).unwrap();

    for argument_struct in parsed_output.argument_structs {
        writeln!(mod_output, "{}", argument_struct).unwrap();
    }

    for fragment in parsed_output.query_fragments {
        writeln!(mod_output, "{}", fragment).unwrap();
    }

    for en in parsed_output.enums {
        writeln!(mod_output, "{}", en).unwrap();
    }

    for input_object in parsed_output.input_objects {
        writeln!(mod_output, "{}", input_object).unwrap();
    }

    for scalar in parsed_output.scalars {
        writeln!(mod_output, "{}", scalar).unwrap();
    }

    writeln!(output, "}}\n").unwrap();

    writeln!(output, "mod {} {{", options.query_module).unwrap();

    writeln!(
        output,
        "    cynic::use_schema!(r#\"{}\"#);",
        options.schema_path
    )
    .unwrap();
    writeln!(output, "}}\n").unwrap();

    Ok(output)
}
