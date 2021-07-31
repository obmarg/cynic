use std::rc::Rc;

mod naming;
mod output;
mod query_parsing;
mod schema;
mod type_ext;

use graphql_parser::error::BorrowedParseError;
use graphql_parser::{ParseError, Pos};
pub use schema::{GraphPath, TypeIndex};

pub use output::indented;
pub use query_parsing::inputs::extract_input_objects;
pub use query_parsing::normalisation::normalise;
pub use query_parsing::parse_query_document;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Query document not supported: {0}")]
    UnsupportedQueryDocument(String),

    #[error("could not parse the document: {0}")]
    ParseError(#[from] ParseError),

    #[error("could not find field `{0}` on `{1}`")]
    UnknownField(String, String, Pos),

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

    #[error("enum-like value `{0}` was provided to an argument that is not an enum")]
    ArgumentNotEnum(String, Pos),

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

impl From<BorrowedParseError<'_>> for Error {
    fn from(error: BorrowedParseError) -> Self {
        Error::ParseError(error.into())
    }
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
        fragment.fmt(mod_output)?;
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::document_to_fragment_structs2;

    #[test]
    fn hehexd() {
        let schema = std::fs::read_to_string("schemas/starwars.schema.graphql")
            .expect("Schema could not be read.");

        let query_string = "query film_directory_query($id: ID) {
            film(id $id) {
                title, #asdasdasd
                director,
            }
        }";

        let fragments = document_to_fragment_structs2(&query_string, &schema, "hehehxd");
    }
}

pub fn document_to_fragment_structs2(
    query: impl AsRef<str>,
    schema: impl AsRef<str>,
    schema_path: impl AsRef<str>,
) -> Result<String, Error> {
    use std::fmt::Write;

    let schema = graphql_parser::parse_schema::<&str>(schema.as_ref())?;
    let query = graphql_parser::parse_query::<&str>(query.as_ref())?;

    // eprintln!("{:#?}", query);

    let type_index = Rc::new(TypeIndex::from_schema(&schema));
    let parsed_output = parse_query_document(&query, &type_index)?;

    let mut output = String::new();

    let operation = parsed_output
        .normalised_document
        .operations
        .first()
        .unwrap();
    let name = operation.name.unwrap();

    writeln!(output, "#[cynic::schema_for_derives(").unwrap();
    writeln!(output, "    file = r#\"{}\"#,", schema_path.as_ref()).unwrap();
    writeln!(output, "    module = \"gql_schema\",").unwrap();
    writeln!(output, ")]\nmod {} {{", name).unwrap();

    let mod_output = &mut indented(&mut output, 4);

    writeln!(mod_output, "use crate::gql_schema;\n").unwrap();

    for argument_struct in &parsed_output.argument_structs {
        writeln!(mod_output, "{}", argument_struct).unwrap();
    }

    for fragment in parsed_output.query_fragments {
        fragment.fmt(mod_output)?;
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

    write!(mod_output, "pub fn query(").unwrap();

    let arguments = if let Some(argument_struct) = parsed_output.argument_structs.iter().next() {
        let mut output = String::new();

        write!(output, "{} {{", argument_struct.name).unwrap();

        for field in &argument_struct.fields {
            write!(mod_output, "{}: {},", field.name(), field.type_spec()).unwrap();
            write!(output, "{},", field.name()).unwrap();
        }

        write!(output, "}}").unwrap();

        output
    } else {
        "()".into()
    };

    writeln!(mod_output, ") -> cynic::GraphQlResponse<{}> {{", name).unwrap();
    writeln!(
        mod_output,
        r#"    use cynic::QueryBuilder;

    let query = {}::build(&{});
    crate::gql_schema::run_query(query)"#,
        name, arguments
    )
    .unwrap();
    writeln!(mod_output, "}}").unwrap();

    writeln!(output, "}}\n").unwrap();

    // eprintln!("{}", output);

    Ok(output)
}
