use graphql_parser::query::{
    Definition, Document, OperationDefinition, Selection, SelectionSet, Value, VariableDefinition,
};
use graphql_parser::schema::{EnumType, Type};

use crate::type_index::ScalarKind;
use crate::{Error, FieldType, TypeExt, TypeIndex};

#[derive(Debug, PartialEq)]
pub struct Field<'a> {
    pub name: &'a str,
    pub field_type: &'a Type<'a, &'a str>,

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

#[derive(Debug, PartialEq)]
pub struct Enum<'a> {
    pub def: &'a EnumType<'a, &'a str>,
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
    type_index: &TypeIndex<'a>,
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
    type_index: &TypeIndex<'a>,
    query_name: Option<&'a str>,
    argument_struct_name: Option<&'b str>,
) -> Result<Vec<PotentialStruct<'a>>, Error> {
    let mut rv = Vec::new();

    let path = &path;

    if !path.is_empty() {
        let type_name = type_index.type_for_path(&path)?.inner_name();
        match type_index.lookup_type(type_name) {
            Some(FieldType::Enum(en)) => return Ok(vec![PotentialStruct::Enum(Enum { def: en })]),
            Some(FieldType::Scalar(ScalarKind::Custom)) => {
                return Ok(vec![PotentialStruct::Scalar(type_name.to_string())]);
            }
            _ => (),
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

                let field_type = type_index.type_for_path(&new_path)?;

                this_fragment.fields.push(Field {
                    name: field.name,
                    field_type,
                    arguments: field.arguments.clone(),
                });

                rv.extend(selection_set_to_structs(
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
        if rv.iter().any(|s| s.uses_arguments()) {
            this_fragment.argument_struct_name = argument_struct_name.map(|name| name.to_string());
        }

        rv.push(PotentialStruct::QueryFragment(this_fragment));
    }

    Ok(rv)
}
