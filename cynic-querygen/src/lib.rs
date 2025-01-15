use std::rc::Rc;

mod casings;
mod naming;
mod output;
mod query_parsing;
mod schema;

use output::Output;
use schema::{add_builtins, GraphPath, TypeIndex};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Query document not supported: {0}")]
    UnsupportedQueryDocument(String),

    #[error("could not parse query document: {0}")]
    QueryParseError(graphql_parser::query::ParseError),

    #[error("could not parse schema document: {0}")]
    SchemaParseError(cynic_parser::Error),

    #[error("could not find field `{0}` on `{1}`")]
    UnknownField(String, String),

    #[error("could not find enum `{0}`")]
    UnknownEnum(String),

    #[error("could not find type `{0}`")]
    UnknownType(String),

    #[error("expected type `{0}` to be an object")]
    ExpectedObject(String),

    #[error("expected type `{0}` to be an object or an interface")]
    ExpectedObjectOrInterface(String),

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

    #[error("expected a homogeneous list of input values")]
    ExpectedHomogenousList,

    #[error("expected field to be a list")]
    ExpectedListType,

    #[error("expected a value that is or contains an input object")]
    ExpectedInputObjectValue,

    #[error("couldn't find a fragment named {0}")]
    UnknownFragment(String),

    #[error("Tried to apply a fragment for a {0} type on a {1} type")]
    TypeConditionFailed(String, String),

    #[error("{0} is not a member of the {1} union type")]
    TypeNotUnionMember(String, String),

    #[error("{0} does not implement the {1} interface")]
    TypeDoesNotImplementInterface(String, String),

    #[error("Could not find a type named {0}, which we expected to be the root type")]
    CouldntFindRootType(String),

    #[error("At least one field should be selected for `{0}`.")]
    NoFieldSelected(String),

    #[error("You tried to select some fields on the type {0} which is not a composite type")]
    TriedToSelectFieldsOfNonComposite(String),

    #[error("An inline fragment on a union or interface type must have a type condition")]
    MissingTypeCondition,
}

#[derive(Debug)]
pub struct QueryGenOptions {
    pub schema_module_name: String,
    pub schema_name: Option<String>,
}

impl Default for QueryGenOptions {
    fn default() -> QueryGenOptions {
        QueryGenOptions {
            schema_module_name: "schema".into(),
            schema_name: None,
        }
    }
}

pub fn document_to_fragment_structs(
    query: impl AsRef<str>,
    schema: impl AsRef<str>,
    options: &QueryGenOptions,
) -> Result<String, Error> {
    use std::fmt::Write;

    let schema = cynic_parser::parse_type_system_document(schema.as_ref())
        .map_err(Error::SchemaParseError)?;

    let query =
        graphql_parser::parse_query::<&str>(query.as_ref()).map_err(Error::QueryParseError)?;

    let (schema, typename_id) = add_builtins(schema);

    let type_index = Rc::new(TypeIndex::from_schema(&schema, typename_id));
    let mut parsed_output = query_parsing::parse_query_document(&query, &type_index)?;

    add_schema_name(&mut parsed_output, options.schema_name.as_deref());

    let mut output = String::new();

    let input_objects_need_lifetime = parsed_output
        .input_objects
        .iter()
        .map(|io| {
            (
                io.name.as_str(),
                io.fields.iter().any(|f| f.type_spec.contains_lifetime_a),
            )
        })
        .collect();
    for variables_struct in parsed_output.variables_structs {
        writeln!(
            output,
            "{}",
            output::VariablesStructForDisplay {
                variables_struct: &variables_struct,
                input_objects_need_lifetime: &input_objects_need_lifetime
            }
        )
        .unwrap();
    }

    for fragment in parsed_output.query_fragments {
        writeln!(output, "{}", fragment).unwrap();
    }

    for fragment in parsed_output.inline_fragments {
        writeln!(output, "{}", fragment).unwrap();
    }

    for en in parsed_output.enums {
        writeln!(output, "{}", en).unwrap();
    }

    for input_object in parsed_output.input_objects {
        writeln!(output, "{}", input_object).unwrap();
    }

    for scalar in parsed_output.scalars {
        writeln!(output, "{}", scalar).unwrap();
    }

    Ok(output)
}

fn add_schema_name(output: &mut Output, schema_name: Option<&str>) {
    let Some(schema_name) = schema_name else {
        return;
    };

    for fragment in &mut output.query_fragments {
        fragment.schema_name = Some(schema_name.to_string());
    }

    for fragment in &mut output.inline_fragments {
        fragment.schema_name = Some(schema_name.to_string());
    }

    for en in &mut output.enums {
        en.schema_name = Some(schema_name.to_string());
    }

    for input_object in &mut output.input_objects {
        input_object.schema_name = Some(schema_name.to_string());
    }

    for scalar in &mut output.scalars {
        scalar.schema_name = Some(schema_name.to_string());
    }
}
