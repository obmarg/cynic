use graphql_parser::query::{
    Definition, Document, OperationDefinition, Selection, SelectionSet, Value, VariableDefinition,
};

use crate::schema::{self, EnumType, ScalarTypeExt, Type, TypeDefinition};
use crate::{Error, TypeExt, TypeIndex};

#[derive(Debug, PartialEq)]
pub struct Field<'a> {
    pub name: &'a str,
    pub field_type: &'a Type<'a>,

    pub arguments: Vec<(&'a str, Value<'a, &'a str>)>,
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
}

impl PotentialStruct<'_> {
    fn uses_arguments(&self) -> bool {
        match self {
            PotentialStruct::QueryFragment(q) => q.fields.iter().any(|f| !f.arguments.is_empty()),
            _ => false,
        }
    }
}

pub fn parse_query_document<'a>(
    doc: &'a Document<'a, &'a str>,
    type_index: &'a TypeIndex<'a>,
) -> Result<Vec<PotentialStruct<'a>>, Error> {
    doc.definitions
        .iter()
        .map(|definition| {
            match definition {
                Definition::Operation(OperationDefinition::Query(query)) => {
                    let mut structs = vec![];

                    let argument_struct_name = if !query.variable_definitions.is_empty() {
                        let argument_struct =
                            ArgumentStruct::from_variables(&query.variable_definitions, query.name);

                        let argument_struct_name = argument_struct.name.clone();

                        structs.push(PotentialStruct::ArgumentStruct(argument_struct));

                        for variable in &query.variable_definitions {
                            if let Some(st) =
                                struct_from_type_name(variable.var_type.inner_name(), type_index)?
                            {
                                structs.push(st)
                            }
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
                Definition::Operation(OperationDefinition::SelectionSet(_)) => {
                    return Err(Error::UnsupportedQueryDocument(format!(
                        "top-level selection sets are not yet supported"
                    )));
                }
                Definition::Fragment(_) => {
                    return Err(Error::UnsupportedQueryDocument(format!(
                        "fragments are not yet supported"
                    )));
                }
            }
        })
        .collect::<Result<Vec<Vec<_>>, Error>>()
        .map(|vec| vec.into_iter().flatten().collect())
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
        if let Some(st) = struct_from_type_name(type_name, type_index)? {
            return Ok(vec![st]);
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
                    arguments: field.arguments.clone(),
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
        _ => Ok(vec![]),
    }
}

fn struct_from_type_name<'a>(
    type_name: &str,
    type_index: &'a TypeIndex<'a>,
) -> Result<Option<PotentialStruct<'a>>, Error> {
    match type_index.lookup_type(type_name) {
        Some(TypeDefinition::Enum(en)) => return Ok(Some(PotentialStruct::Enum(Enum { def: en }))),
        Some(TypeDefinition::Scalar(scalar_type)) => {
            if !scalar_type.is_builtin() {
                Ok(Some(PotentialStruct::Scalar(type_name.to_string())))
            } else {
                // We don't create structs for built in scalars
                Ok(None)
            }
        }
        None => Err(Error::UnknownType(type_name.to_string())),
        _ => Ok(None),
    }
}
