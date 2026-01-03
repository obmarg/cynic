use std::{collections::HashMap, rc::Rc};

mod casings;
mod naming;
mod output;
mod query_parsing;
mod schema;

use cynic_parser::{SchemaCoordinate, type_system::ids::FieldDefinitionId};
use output::Output;
use schema::{GraphPath, TypeIndex, add_builtins};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Query document not supported: {0}")]
    UnsupportedQueryDocument(String),

    #[error("could not parse query document: {0}")]
    QueryParseError(cynic_parser::Error),

    #[error("could not parse schema document: {0}")]
    SchemaParseError(cynic_parser::Error),

    #[error("could not find field `{0}` on `{1}`")]
    UnknownField(String, String),

    #[error("could not find enum `{0}`")]
    UnknownEnum(String),

    #[error("could not find type `{0}`")]
    UnknownType(String),

    #[error("could not find directive `{0}`")]
    UnknownDirective(String),

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

pub struct Generator {
    /// The name of a registered schema to use inside generated `#[cynic(schema = "schema_name")]` attributes.
    schema_name: Option<String>,
    /// The parsed schema that will be used to generate documents
    schema: cynic_parser::TypeSystemDocument,
    /// The FieldDefinitionId of the __typename field in the schema.
    typename_id: FieldDefinitionId,
    /// Mapping of `("TypeName.fieldName", "fully::qualified::type::Path")` overrides to customize the scalar type used for specific fields.
    ///
    /// The field name should have the same casing as the field name in the GraphQL schema
    /// (i.e. before conversion to snake_case). Note that the provided type override will still
    /// be wrapped in an [`Option`] if the field is nullable in the schema.
    ///
    /// The override type must be a registered [custom scalar](https://cynic-rs.dev/derives/scalars#custom-scalars) for the schema scalar type
    /// of the overridden field.
    overrides: OverrideMap,
}

type OverrideMap = HashMap<SchemaCoordinate, String>;

impl Generator {
    pub fn new(schema: impl AsRef<str>) -> Result<Self, SchemaParseError> {
        let schema = cynic_parser::parse_type_system_document(schema.as_ref())?;
        let (schema, typename_id) = add_builtins(schema);

        Ok(Generator {
            schema_name: None,
            schema,
            typename_id,
            overrides: HashMap::default(),
        })
    }

    /// Provides the name of a registered schema to use inside generated `#[cynic(schema = "schema_name")]` attributes.
    pub fn with_schema_name(mut self, schema_name: impl Into<String>) -> Self {
        self.schema_name = Some(schema_name.into());
        self
    }

    /// Provides the name of a registered schema to use inside generated `#[cynic(schema = "schema_name")]` attributes.
    pub fn set_schema_name(&mut self, schema_name: impl Into<String>) {
        self.schema_name = Some(schema_name.into());
    }

    /// Sets an override to the code that this `Generator` will generate.
    ///
    /// The `coordinate` argument should be set to a valid [schema coordinate][1] - note that
    /// currently only member coordinates are currently supported by the generator.
    ///
    /// ### Member Coordinate Behaviour
    ///
    /// The field name should have the same casing as the field name in the GraphQL schema (i.e.
    /// before conversion to snake_case). Note that the provided type override will still be
    /// wrapped in an [`Option`] if the field is nullable in the schema.
    ///
    /// The replacement type must be a registered [custom scalar][2] for the schema scalar type of
    /// the overridden field.
    ///
    /// [1]: https://spec.graphql.org/September2025/#sec-Schema-Coordinates
    /// [2]: https://cynic-rs.dev/derives/scalars#custom-scalars
    pub fn with_override(
        mut self,
        coordinate: impl AsRef<str>,
        replacement: impl Into<String>,
    ) -> Result<Self, InvalidSchemaCoordinate> {
        self.set_override(coordinate, replacement)?;

        Ok(self)
    }

    /// Sets many overrides to the code that this `Generator` will generate.
    ///
    /// See [`Generator::with_override`] for more details on overrides.
    pub fn with_overrides<Iter, Coord, Replacement>(
        mut self,
        iter: Iter,
    ) -> Result<Self, InvalidSchemaCoordinate>
    where
        Iter: IntoIterator<Item = (Coord, Replacement)>,
        Coord: AsRef<str>,
        Replacement: Into<String>,
    {
        for (coordinate, replacement) in iter.into_iter() {
            self.set_override(coordinate, replacement)?;
        }

        Ok(self)
    }

    /// Sets an override in the code that this `Generator` will generate.
    ///
    /// The `coordinate` argument should be set to a valid [schema coordinate][1] - note that
    /// currently only member coordinates are currently supported by the generator.
    ///
    /// ### Member Coordinate Behaviour
    ///
    /// The field name should have the same casing as the field name in the GraphQL schema (i.e.
    /// before conversion to snake_case). Note that the provided type override will still be
    /// wrapped in an [`Option`] if the field is nullable in the schema.
    ///
    /// The replacement type must be a registered [custom scalar][2] for the schema scalar type of
    /// the overridden field.
    ///
    /// [1]: https://spec.graphql.org/September2025/#sec-Schema-Coordinates
    /// [2]: https://cynic-rs.dev/derives/scalars#custom-scalars
    pub fn set_override(
        &mut self,
        coordinate: impl AsRef<str>,
        replacement: impl Into<String>,
    ) -> Result<(), InvalidSchemaCoordinate> {
        let coordinate = cynic_parser::parse_schema_coordinate(coordinate.as_ref())?;

        if !coordinate.is_member() {
            return Err(InvalidSchemaCoordinate::UnsupportedCoordinate(coordinate));
        }

        self.overrides.insert(coordinate, replacement.into());

        Ok(())
    }

    /// Sets many overrides to the code that this `Generator` will generate.
    ///
    /// See [`Generator::set_override`] for more details on overrides.
    pub fn set_overrides<Iter, Coord, Replacement>(
        &mut self,
        iter: Iter,
    ) -> Result<(), InvalidSchemaCoordinate>
    where
        Iter: IntoIterator<Item = (Coord, Replacement)>,
        Coord: AsRef<str>,
        Replacement: Into<String>,
    {
        for (coordinate, replacement) in iter.into_iter() {
            self.set_override(coordinate, replacement)?;
        }

        Ok(())
    }

    /// Generates rust code for the provided query
    pub fn generate(&self, query: impl AsRef<str>) -> Result<String, Error> {
        fn generate_impl(generator: &Generator, query: &str) -> Result<String, Error> {
            use std::fmt::Write;

            let query =
                cynic_parser::parse_executable_document(query).map_err(Error::QueryParseError)?;

            let type_index = Rc::new(TypeIndex::from_schema(
                &generator.schema,
                generator.typename_id,
            ));
            let mut parsed_output =
                query_parsing::parse_query_document(&query, &type_index, &generator.overrides)?;

            add_schema_name(&mut parsed_output, generator.schema_name.as_deref());

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
        generate_impl(self, query.as_ref())
    }
}

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub struct SchemaParseError(#[from] cynic_parser::Error);

#[derive(thiserror::Error, Debug)]
pub enum InvalidSchemaCoordinate {
    #[error(transparent)]
    ParseError(#[from] cynic_parser::Error),
    #[error("Unsupported schema coordinate: {0}")]
    UnsupportedCoordinate(SchemaCoordinate),
}

#[derive(Debug)]
#[deprecated(since = "3.14.0", note = "use Generator instead")]
pub struct QueryGenOptions {
    pub schema_module_name: String,
    /// The name of a registered schema to use inside generated `#[cynic(schema = "schema_name")]` attributes.
    pub schema_name: Option<String>,
}

#[expect(deprecated)]
impl Default for QueryGenOptions {
    fn default() -> QueryGenOptions {
        QueryGenOptions {
            schema_module_name: "schema".into(),
            schema_name: None,
        }
    }
}

#[expect(deprecated)]
#[deprecated(since = "3.14.0", note = "use Generator::xxx instead")]
pub fn document_to_fragment_structs(
    query: impl AsRef<str>,
    schema: impl AsRef<str>,
    options: &QueryGenOptions,
) -> Result<String, Error> {
    let mut generator = Generator::new(schema).map_err(|e| Error::SchemaParseError(e.0))?;
    if let Some(schema_name) = &options.schema_name {
        generator.set_schema_name(schema_name);
    }
    // TODO: use options
    generator.generate(query)
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
