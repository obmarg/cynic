use std::{
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

use super::{
    normalisation::{NormalisedDocument, Selection, SelectionSet},
    sorting::Vertex,
    value::Value,
};
use crate::{
    schema::{self, TypeDefinition},
    type_ext::TypeExt,
    Error, TypeIndex,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InputObject {
    target_type: String,
    fields: BTreeMap<String, InputObjectField>,
}

impl InputObject {
    /// Extracts any named types used by this InputObject
    pub fn named_types(&self) -> impl Iterator<Item = &str> {
        self.fields.iter().flat_map(|(_, field)| match field {
            InputObjectField::NamedType(name) => Some(name.as_ref()),
            _ => None,
        })
    }
}

impl Vertex for InputObject {
    fn adjacents(self: &Rc<InputObject>) -> Vec<Rc<InputObject>> {
        self.fields
            .iter()
            .flat_map(|(_, field)| match field {
                InputObjectField::Object(other_obj) => Some(Rc::clone(other_obj)),
                _ => None,
            })
            .collect()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InputObjectField {
    Object(Rc<InputObject>),
    NamedType(String),
}

type InputObjectSet = BTreeSet<Rc<InputObject>>;

pub fn extract_input_objects<'query, 'schema>(
    doc: &NormalisedDocument<'query, '_>,
    type_index: &TypeIndex<'schema>,
) -> Result<InputObjectSet, Error> {
    let mut result = InputObjectSet::new();

    // Walk the selection sets looking for input objects
    for selection_set in &doc.selection_sets {
        extract_objects_from_selection_set(&selection_set, type_index, &mut result)?;
    }

    // Find any query variables that are input objects
    for operation in &doc.operations {
        for variable in &operation.variable_definitions {
            let variable_type_name = variable.var_type.inner_name();
            let variable_type = type_index
                .lookup_type(variable_type_name)
                .ok_or_else(|| Error::UnknownType(variable_type_name.to_string()))?;

            if let TypeDefinition::InputObject(input_obj) = variable_type {
                extract_whole_input_object(input_obj, type_index, &mut result)?;
            }
        }
    }

    Ok(result)
}

fn extract_objects_from_selection_set<'query, 'schema>(
    selection_set: &Rc<SelectionSet<'query>>,
    type_index: &TypeIndex<'schema>,
    input_objects: &mut InputObjectSet,
) -> Result<(), Error> {
    if selection_set.selections.is_empty() {
        return Ok(());
    }

    let object_definition = if let Some(TypeDefinition::Object(object_def)) =
        type_index.lookup_type(&selection_set.target_type)
    {
        object_def
    } else {
        return Err(Error::ExpectedObject(selection_set.target_type.to_string()));
    };

    for selection in &selection_set.selections {
        match selection {
            Selection::Field(field) => {
                extract_objects_from_selection_set(
                    &field.selection_set,
                    type_index,
                    input_objects,
                )?;

                let field_definition = object_definition
                    .fields
                    .iter()
                    .find(|f| f.name == field.name)
                    .ok_or_else(|| {
                        Error::UnknownField(
                            field.name.to_string(),
                            selection_set.target_type.to_string(),
                        )
                    })?;

                for (arg_name, arg_value) in &field.arguments {
                    let arg_definition = field_definition
                        .arguments
                        .iter()
                        .find(|v| v.name == *arg_name)
                        .ok_or_else(|| Error::UnknownArgument(arg_name.to_string()))?;

                    let arg_type_name = arg_definition.value_type.inner_name();
                    let arg_type = type_index
                        .lookup_type(&arg_type_name)
                        .ok_or_else(|| Error::UnknownType(arg_type_name.to_string()))?;

                    let input_obj = if let TypeDefinition::InputObject(input_obj) = arg_type {
                        input_obj
                    } else {
                        continue;
                    };

                    extract_input_objects_from_values(
                        input_obj,
                        arg_value,
                        type_index,
                        input_objects,
                    )?;
                }
            }
        }
    }

    Ok(())
}

pub fn extract_input_objects_from_values<'schema, 'query>(
    input_object: &schema::InputObjectType,
    value: &Value<'query>,
    type_index: &TypeIndex<'schema>,
    input_objects: &mut InputObjectSet,
) -> Result<Rc<InputObject>, Error> {
    match value {
        Value::Variable(_) => {
            extract_whole_input_object(input_object, type_index, input_objects)
        }
        Value::Object(obj) => {
            let mut fields = BTreeMap::new();
            for (field_name, field_val) in obj {
                let field = input_object
                    .fields
                    .iter()
                    .find(|f| f.name == *field_name)
                    .ok_or_else(|| {
                        Error::UnknownField(field_name.to_string(), input_object.name.to_string())
                    })?;

                let field_type_name = field.value_type.inner_name();
                let field_type = type_index
                    .lookup_type(&field_type_name)
                    .ok_or_else(|| Error::UnknownType(field_type_name.to_string()))?;

                let field_out_val = match field_type {
                    TypeDefinition::InputObject(inner_obj) => {
                        InputObjectField::Object(extract_input_objects_from_values(
                            inner_obj,
                            field_val,
                            type_index,
                            input_objects,
                        )?)
                    }
                    TypeDefinition::Scalar(scalar) => {
                        InputObjectField::NamedType(scalar.name.to_string())
                    }
                    TypeDefinition::Enum(en) => InputObjectField::NamedType(en.name.to_string()),
                    _ => return Err(Error::ExpectedInputType),
                };

                fields.insert(field_name.to_string(), field_out_val);
            }

            let rv = Rc::new(InputObject {
                target_type: input_object.name.to_string(),
                fields,
            });

            if let Some(existing_obj) = input_objects.get(&rv) {
                return Ok(Rc::clone(existing_obj));
            }

            input_objects.insert(Rc::clone(&rv));

            Ok(rv)
        }
        Value::List(inner_values) => {
            if inner_values.is_empty() {
                // We still need the type in order to type this field...
                return extract_whole_input_object(input_object, type_index, input_objects);
            }

            let mut output_values = Vec::with_capacity(inner_values.len());
            for inner_value in inner_values {
                output_values.push(extract_input_objects_from_values(
                    input_object,
                    inner_value,
                    type_index,
                    input_objects,
                )?);
            }

            let mut output_iter = output_values.into_iter();
            let rv = output_iter.next().unwrap();
            if output_iter.any(|v| v != rv) {
                return Err(Error::ExpectedHomogenousList);
            }

            Ok(rv)
        }
        _ => Err(Error::ExpectedInputObjectValue),
    }
}

pub fn extract_whole_input_object<'schema>(
    input_object: &schema::InputObjectType,
    type_index: &TypeIndex<'schema>,
    input_objects: &mut InputObjectSet,
) -> Result<Rc<InputObject>, Error> {
    let mut fields = BTreeMap::new();

    for field in &input_object.fields {
        let field_type_name = field.value_type.inner_name();
        let field_type = type_index
            .lookup_type(field_type_name)
            .ok_or_else(|| Error::UnknownType(field_type_name.to_string()))?;

        let field_out_val = match field_type {
            TypeDefinition::InputObject(inner_obj) => InputObjectField::Object(
                extract_whole_input_object(inner_obj, type_index, input_objects)?,
            ),
            TypeDefinition::Scalar(scalar) => InputObjectField::NamedType(scalar.name.to_string()),
            TypeDefinition::Enum(en) => InputObjectField::NamedType(en.name.to_string()),
            _ => return Err(Error::ExpectedInputType),
        };

        fields.insert(field.name.to_string(), field_out_val);
    }

    let rv = Rc::new(InputObject {
        target_type: input_object.name.to_string(),
        fields,
    });

    if let Some(existing_obj) = input_objects.get(&rv) {
        return Ok(Rc::clone(existing_obj));
    }

    input_objects.insert(Rc::clone(&rv));

    Ok(rv)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query_parsing_mk2::normalisation::normalise;

    #[test]
    fn extracts_inline_input_types() {
        let schema = load_schema();
        let type_index = TypeIndex::from_schema(&schema);
        let query = graphql_parser::parse_query::<&str>(
            r#"
              query {
                cynic: repository(owner: "obmarg", name: "cynic") {
                  issues(filterBy: {labels: ["good first issue"]}) {
                    nodes {
                      title
                    }
                  }
                }
              	kazan: repository(owner: "obmarg", name: "kazan") {
                  issues(filterBy: {labels: ["good first issue"], mentioned: "obmarg"}) {
                    nodes {
                      title
                   }
                  }
                }
              }
            "#,
        )
        .unwrap();

        let normalised = normalise(&query, &type_index).unwrap();
        let input_objects = extract_input_objects(&normalised, &type_index).unwrap();

        assert_eq!(input_objects.len(), 2);
    }

    #[test]
    fn deduplicates_input_types_if_same() {
        let schema = load_schema();
        let type_index = TypeIndex::from_schema(&schema);
        let query = graphql_parser::parse_query::<&str>(
            r#"
              query {
                cynic: repository(owner: "obmarg", name: "cynic") {
                  issues(filterBy: {labels: ["good first issue"]}) {
                    nodes {
                      title
                    }
                  }
                }
              	kazan: repository(owner: "obmarg", name: "kazan") {
                  issues(filterBy: {labels: ["good first issue"]}) {
                    nodes {
                      title
                   }
                  }
                }
              }
            "#,
        )
        .unwrap();

        let normalised = normalise(&query, &type_index).unwrap();
        let input_objects = extract_input_objects(&normalised, &type_index).unwrap();

        assert_eq!(input_objects.len(), 1);
    }

    #[test]
    fn finds_variable_input_types() {
        let schema = load_schema();
        let type_index = TypeIndex::from_schema(&schema);
        let query = graphql_parser::parse_query::<&str>(
            r#"
              query MyQuery($input: IssueFilters!) {
                cynic: repository(owner: "obmarg", name: "cynic") {
                  issues(filterBy: $issueFilters) {
                    nodes {
                      title
                    }
                  }
                }
              	kazan: repository(owner: "obmarg", name: "kazan") {
                  issues(filterBy: $issueFilters) {
                    nodes {
                      title
                   }
                  }
                }
              }
            "#,
        )
        .unwrap();

        let normalised = normalise(&query, &type_index).unwrap();
        let input_objects = extract_input_objects(&normalised, &type_index).unwrap();

        assert_eq!(input_objects.len(), 1);
    }

    fn load_schema() -> schema::Document<'static> {
        graphql_parser::parse_schema::<&str>(include_str!("../../tests/schemas/github.graphql"))
            .unwrap()
    }
}
