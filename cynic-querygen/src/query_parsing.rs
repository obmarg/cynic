use graphql_parser::query::{
    Definition, Document, OperationDefinition, Selection, SelectionSet, Value, VariableDefinition,
};

use crate::schema::{self, EnumType, InputValue, ScalarTypeExt, Type, TypeDefinition};
use crate::{value_ext::ValueExt, Error, TypeExt, TypeIndex};

#[derive(Debug, PartialEq)]
pub struct Field<'a> {
    pub name: &'a str,
    pub field_type: &'a Type<'a>,

    pub arguments: Vec<FieldArgument<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct FieldArgument<'a> {
    pub name: &'a str,
    value: Value<'a, &'a str>,
    argument_type: &'a TypeDefinition<'a>,
}

impl<'a> FieldArgument<'a> {
    fn new(
        name: &'a str,
        value: Value<'a, &'a str>,
        argument_type: &'a TypeDefinition<'a>,
    ) -> Self {
        FieldArgument {
            name,
            value,
            argument_type,
        }
    }

    pub fn to_literal(&self, type_index: &TypeIndex) -> Result<String, Error> {
        self.value.to_literal(self.argument_type, type_index)
    }
}

#[derive(Debug, PartialEq)]
pub struct QueryFragment<'a> {
    pub fields: Vec<Field<'a>>,
    pub path: Vec<&'a str>,

    pub argument_struct_name: Option<String>,

    // QueryFragments get the query name if they're at the root of a query
    pub name: Option<&'a str>,
}

impl QueryFragment<'_> {
    fn uses_arguments(&self) -> bool {
        self.fields.iter().any(|f| !f.arguments.is_empty())
    }
}

#[derive(Debug, PartialEq)]
pub struct Enum<'a> {
    pub def: &'a EnumType<'a>,
}

#[derive(Debug, PartialEq)]
pub struct ArgumentStruct<'a> {
    pub name: String,
    pub fields: Vec<Field<'a>>,
}

impl<'a> ArgumentStruct<'a> {
    fn from_variables(
        variables: &'a Vec<VariableDefinition<'a, &'a str>>,
        query_name: Option<&'a str>,
    ) -> ArgumentStruct<'a> {
        ArgumentStruct {
            name: format!("{}Arguments", query_name.unwrap_or("")),
            fields: variables
                .iter()
                .map(|var| Field {
                    name: var.name,
                    field_type: &var.var_type,
                    arguments: vec![],
                })
                .collect(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum PotentialStruct<'a> {
    QueryFragment(QueryFragment<'a>),
    Enum(Enum<'a>),
    Scalar(String),
    ArgumentStruct(ArgumentStruct<'a>),
    InputObject(InputObject<'a>),
}

impl PotentialStruct<'_> {
    fn uses_arguments(&self) -> bool {
        match self {
            PotentialStruct::QueryFragment(q) => q.fields.iter().any(|f| !f.arguments.is_empty()),
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct InputObject<'a> {
    pub name: String,
    pub fields: Vec<InputValue<'a>>,
}

pub fn parse_query_document<'a>(
    doc: &'a Document<'a, &'a str>,
    type_index: &'a TypeIndex<'a>,
) -> Result<Vec<PotentialStruct<'a>>, Error> {
    doc.definitions
        .iter()
        .map(|definition| parse_definition(&definition, type_index))
        .collect::<Result<Vec<Vec<_>>, Error>>()
        .map(|vec| vec.into_iter().flatten().collect())
}

fn parse_definition<'a>(
    definition: &'a Definition<'a, &'a str>,
    type_index: &'a TypeIndex<'a>,
) -> Result<Vec<PotentialStruct<'a>>, Error> {
    match definition {
        Definition::Operation(OperationDefinition::Query(query)) => {
            let mut structs = vec![];

            let argument_struct_name = if !query.variable_definitions.is_empty() {
                let argument_struct =
                    ArgumentStruct::from_variables(&query.variable_definitions, query.name);

                let argument_struct_name = argument_struct.name.clone();

                structs.push(PotentialStruct::ArgumentStruct(argument_struct));

                for variable in &query.variable_definitions {
                    structs.extend(structs_from_type_name(
                        variable.var_type.inner_name(),
                        type_index,
                    )?);
                }

                Some(argument_struct_name)
            } else {
                None
            };

            let mut selection_structs = selection_set_to_structs(
                &query.selection_set,
                vec![],
                type_index,
                query.name,
                argument_struct_name.as_deref(),
            )?;

            // selection_set_to_structs traverses the tree in post-order
            // (sort of), so we reverse to get the root node first.
            selection_structs.reverse();

            structs.append(&mut selection_structs);

            Ok(structs)
        }
        Definition::Operation(OperationDefinition::Mutation(_)) => {
            return Err(Error::UnsupportedQueryDocument(format!(
                "mutations are not yet supported"
            )));
        }
        Definition::Operation(OperationDefinition::Subscription(_)) => {
            return Err(Error::UnsupportedQueryDocument(format!(
                "subscriptions are not supported"
            )));
        }
        Definition::Operation(OperationDefinition::SelectionSet(selection_set)) => {
            let mut selection_structs = selection_set_to_structs(
                &selection_set,
                vec![],
                type_index,
                Some("UnnamedQuery"),
                None,
            )?;

            // selection_set_to_structs traverses the tree in post-order
            // (sort of), so we reverse to get the root node first.
            selection_structs.reverse();

            Ok(selection_structs)
        }
        Definition::Fragment(_) => {
            return Err(Error::UnsupportedQueryDocument(format!(
                "fragments are not yet supported"
            )));
        }
    }
}

fn selection_set_to_structs<'a, 'b>(
    selection_set: &'a SelectionSet<'a, &'a str>,
    path: Vec<&'a str>,
    type_index: &'a TypeIndex<'a>,
    query_name: Option<&'a str>,
    argument_struct_name: Option<&'b str>,
) -> Result<Vec<PotentialStruct<'a>>, Error> {
    let mut nested_types = Vec::new();

    let path = &path;

    if !path.is_empty() {
        let type_name = type_index.field_for_path(&path)?.field_type.inner_name();
        let structs = structs_from_type_name(type_name, type_index)?;
        if !structs.is_empty() {
            return Ok(structs);
        }
    }

    let mut this_fragment = QueryFragment {
        path: path.clone(),
        fields: Vec::new(),
        name: query_name,
        argument_struct_name: None,
    };

    for item in &selection_set.items {
        match item {
            Selection::Field(field) => {
                let mut new_path = path.clone();
                new_path.push(field.name);

                let schema_field = type_index.field_for_path(&new_path)?;

                this_fragment.fields.push(Field {
                    name: field.name,
                    field_type: &schema_field.field_type,
                    arguments: field
                        .arguments
                        .iter()
                        .map(|(name, value)| -> Result<FieldArgument, Error> {
                            let argument = schema_field
                                .arguments
                                .iter()
                                .find(|arg| &arg.name == name)
                                .ok_or(Error::UnknownArgument(name.to_string()))?;

                            let argument_type = type_index
                                .lookup_type(argument.value_type.inner_name())
                                .ok_or(Error::UnknownType(
                                    argument.value_type.inner_name().to_string(),
                                ))?;

                            Ok(FieldArgument::new(name, value.clone(), argument_type))
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                });

                nested_types.extend(
                    field
                        .arguments
                        .iter()
                        .map(|(name, value)| {
                            argument_to_structs(name, value, schema_field, type_index)
                        })
                        .collect::<Result<Vec<_>, _>>()?
                        .into_iter()
                        .flatten(),
                );

                nested_types.extend(selection_set_to_structs(
                    &field.selection_set,
                    new_path,
                    type_index,
                    None,
                    argument_struct_name,
                )?);
            }
            Selection::FragmentSpread(_) => {
                return Err(Error::UnsupportedQueryDocument(
                    "Fragment spreads are not yet supported".into(),
                ));
            }
            Selection::InlineFragment(_) => {
                return Err(Error::UnsupportedQueryDocument(
                    "Inline fragments are not yet supported".into(),
                ));
            }
        }
    }

    if !this_fragment.fields.is_empty() {
        if this_fragment.uses_arguments() || nested_types.iter().any(|s| s.uses_arguments()) {
            this_fragment.argument_struct_name = argument_struct_name.map(|name| name.to_string());
        }

        nested_types.push(PotentialStruct::QueryFragment(this_fragment));
    }

    Ok(nested_types)
}

fn argument_to_structs<'a>(
    arg_name: &'a str,
    arg_value: &Value<'a, &'a str>,
    schema_field: &'a schema::Field<'a>,
    type_index: &'a TypeIndex<'a>,
) -> Result<Vec<PotentialStruct<'a>>, Error> {
    let schema_argument = schema_field
        .arguments
        .iter()
        .find(|arg| arg.name == arg_name)
        .ok_or(Error::UnknownArgument(arg_name.to_string()))?;

    match arg_value {
        Value::Enum(_) => {
            let enum_name = schema_argument.value_type.inner_name();

            if let Some(TypeDefinition::Enum(en)) = type_index.lookup_type(enum_name) {
                Ok(vec![PotentialStruct::Enum(Enum { def: en })])
            } else {
                Err(Error::UnknownEnum(enum_name.to_string()))
            }
        }
        Value::Object(_) => {
            // This is probably an InputObject, so lets extract the
            // InputObject & any other types it contains
            Ok(structs_from_type_name(
                schema_argument.value_type.inner_name(),
                type_index,
            )?)
        }
        _ => Ok(vec![]),
    }
}

fn structs_from_type_name<'a>(
    type_name: &str,
    type_index: &'a TypeIndex<'a>,
) -> Result<Vec<PotentialStruct<'a>>, Error> {
    match type_index.lookup_type(type_name) {
        Some(TypeDefinition::Enum(en)) => return Ok(vec![PotentialStruct::Enum(Enum { def: en })]),
        Some(TypeDefinition::Scalar(scalar_type)) => {
            if !scalar_type.is_builtin() {
                Ok(vec![PotentialStruct::Scalar(type_name.to_string())])
            } else {
                // We don't create structs for built in scalars
                Ok(vec![])
            }
        }
        Some(TypeDefinition::InputObject(input_object)) => {
            let mut rv = Vec::new();
            rv.push(PotentialStruct::InputObject(InputObject {
                name: type_name.to_string(),
                fields: input_object.fields.clone(),
            }));

            rv.extend(
                input_object
                    .fields
                    .iter()
                    .map(|field| structs_from_type_name(field.value_type.inner_name(), type_index))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_iter()
                    .flatten(),
            );

            Ok(rv)
        }
        None => Err(Error::UnknownType(type_name.to_string())),
        _ => Ok(vec![]),
    }
}
