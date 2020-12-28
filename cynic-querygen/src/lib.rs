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
    let schema = graphql_parser::parse_schema::<&str>(schema.as_ref())?;
    let query = graphql_parser::parse_query::<&str>(query.as_ref())?;

    let type_index = Rc::new(TypeIndex::from_schema(&schema));
    let output = query_parsing::parse_query_document(&query, &type_index)?;

    let mut lines = vec![];

    lines.push("#[cynic::query_module(".into());
    lines.push(format!("    schema_path = r#\"{}\"#,", options.schema_path));
    lines.push(format!("    query_module = \"{}\",", options.query_module));
    lines.push(")]\nmod queries {".into());
    lines.push(format!(
        "    use super::{{{}, types::*}};\n",
        options.query_module
    ));

    for argument_struct in output.argument_structs {
        use output::indented;
        use std::fmt::Write;

        let mut s = String::new();
        write!(indented(&mut s, 4), "{}", argument_struct).unwrap();
        lines.push(s);
    }

    for fragment in output.query_fragments {
        use output::indented;
        use std::fmt::Write;

        let mut s = String::new();
        write!(indented(&mut s, 4), "{}", fragment).unwrap();
        lines.push(s);
    }

    for en in output.enums {
        use output::indented;
        use std::fmt::Write;

        let mut s = String::new();
        write!(indented(&mut s, 4), "{}", en).unwrap();
        lines.push(s);
    }

    for input_object in output.input_objects {
        use output::indented;
        use std::fmt::Write;

        let mut s = String::new();
        write!(indented(&mut s, 4), "{}", input_object).unwrap();
        lines.push(s);
    }
    lines.push("}\n".into());

    lines.push("#[cynic::query_module(".into());
    lines.push(format!("    schema_path = r#\"{}\"#,", options.schema_path));
    lines.push(format!("    query_module = \"{}\",", options.query_module));
    lines.push(")]\nmod types {".into());

    // Output any custom scalars we need.
    // Note that currently our query_dsl needs _all_ scalars in a schema
    // so we're parsing this out from schema.definitons rather than output.scalars
    for def in &schema.definitions {
        if let schema::Definition::TypeDefinition(schema::TypeDefinition::Scalar(scalar)) = def {
            lines.push("    #[derive(cynic::Scalar, Debug)]".into());
            lines.push(format!(
                "    pub struct {}(String);\n",
                scalar.name.to_pascal_case()
            ));
        }
    }
    lines.push("}\n".into());

    lines.push(format!("mod {}{{", options.query_module));
    lines.push("    use super::types::*;".into());
    lines.push(format!(
        "    cynic::query_dsl!(r#\"{}\"#);",
        options.schema_path
    ));
    lines.push("}\n".into());

    Ok(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
