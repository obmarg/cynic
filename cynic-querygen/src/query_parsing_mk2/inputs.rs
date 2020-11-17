use std::{
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

use super::{
    normalisation::{Field, NormalisedDocument, Selection, SelectionSet},
    sorting::Vertex,
    value::TypedValue,
};
use crate::{
    schema::{self, InputFieldType, InputType, InputTypeRef},
    Error,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InputObject<'schema> {
    pub schema_type: schema::InputObjectDetails<'schema>,
    pub fields: Vec<schema::InputField<'schema>>,
    _adjacents: Vec<Rc<InputObject<'schema>>>,
}

impl<'schema> InputObject<'schema> {
    /// Extracts any input types used by this InputObject
    pub fn required_input_types(&self) -> Vec<InputTypeRef<'schema>> {
        self.fields
            .iter()
            .map(|field| field.value_type.inner_ref().clone())
            .collect()
    }
}

impl<'schema> Vertex for InputObject<'schema> {
    fn adjacents(self: &Rc<InputObject<'schema>>) -> Vec<Rc<InputObject<'schema>>> {
        self._adjacents.clone()
    }
}

pub type InputObjectSet<'schema> = BTreeSet<Rc<InputObject<'schema>>>;

pub fn extract_input_objects<'query, 'schema>(
    doc: &NormalisedDocument<'query, 'schema>,
) -> Result<InputObjectSet<'schema>, Error> {
    let mut result = InputObjectSet::new();

    // Walk the selection sets looking for input objects
    for selection_set in &doc.selection_sets {
        extract_objects_from_selection_set(&selection_set, &mut result)?;
    }

    // Find any query variables that are input objects
    for operation in &doc.operations {
        for variable in &operation.variables {
            let variable_type = variable.value_type.inner_ref().lookup()?;

            if let InputType::InputObject(input_obj) = variable_type {
                extract_whole_input_object(&input_obj, &mut result)?;
            }
        }
    }

    Ok(result)
}

fn extract_objects_from_selection_set<'query, 'schema>(
    selection_set: &Rc<SelectionSet<'query, 'schema>>,
    input_objects: &mut InputObjectSet<'schema>,
) -> Result<(), Error> {
    if selection_set.selections.is_empty() {
        return Ok(());
    }

    for selection in &selection_set.selections {
        match selection {
            Selection::Field(field) => {
                let selection_set = if let Field::Composite(ss) = &field.field {
                    ss
                } else {
                    continue;
                };

                extract_objects_from_selection_set(selection_set, input_objects)?;

                for (_, arg_value) in &field.arguments {
                    let arg_type = arg_value.value_type().inner_ref().lookup()?;

                    if let InputType::InputObject(input_obj) = arg_type {
                        extract_input_objects_from_values(&input_obj, arg_value, input_objects)?;
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn extract_input_objects_from_values<'query, 'schema>(
    input_object: &schema::InputObjectDetails<'schema>,
    typed_value: &TypedValue<'query, 'schema>,
    input_objects: &mut InputObjectSet<'schema>,
) -> Result<Rc<InputObject<'schema>>, Error> {
    if typed_value.is_variable() {
        return extract_whole_input_object(input_object, input_objects);
    }

    match &typed_value {
        TypedValue::Object(obj, _) => {

            let mut fields = Vec::new();
            let mut adjacents = Vec::new();
            for (field_name, field_val) in obj {
                let field = input_object
                    .fields
                    .iter()
                    .find(|f| f.name == *field_name)
                    .ok_or_else(|| {
                        Error::UnknownField(field_name.to_string(), input_object.name.to_string())
                    })?;

                let field_type = field.value_type.inner_ref().lookup()?;

                match field_type {
                    InputType::InputObject(inner_obj) => {
                        adjacents.push(extract_input_objects_from_values(
                            &inner_obj,
                            field_val,
                            input_objects,
                        )?);
                    }
                    _ => {}
                }

                fields.push(field.clone());
            }

            let rv = Rc::new(InputObject {
                schema_type: input_object.clone(),
                _adjacents: adjacents,
                fields,
            });

            if let Some(existing_obj) = input_objects.get(&rv) {
                return Ok(Rc::clone(existing_obj));
            }

            input_objects.insert(Rc::clone(&rv));

            Ok(rv)
        }
        TypedValue::List(inner_values, _) => {
            // TODO: Consider re-working this...

            if inner_values.is_empty() {
                // We still need the type in order to type this field...
                return extract_whole_input_object(input_object, input_objects);
            }

            let inner_type = if let InputFieldType::ListType(inner_ty) = &typed_value.value_type() {
                inner_ty
            } else {
                return Err(Error::ExpectedListType);
            };

            let mut output_values = Vec::with_capacity(inner_values.len());
            for inner_value in inner_values {
                output_values.push(extract_input_objects_from_values(
                    input_object,
                    &inner_value,
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
    input_object: &schema::InputObjectDetails<'schema>,
    input_objects: &mut InputObjectSet<'schema>,
) -> Result<Rc<InputObject<'schema>>, Error> {
    let mut fields = Vec::new();
    let mut adjacents = Vec::new();

    for field in &input_object.fields {
        let field_type = field.value_type.inner_ref().lookup()?;

        match field_type {
            InputType::InputObject(inner_obj) => {
                adjacents.push(extract_whole_input_object(&inner_obj, input_objects)?);
            }
            _ => {}
        }

        fields.push(field.clone());
    }

    let rv = Rc::new(InputObject {
        schema_type: input_object.clone(),
        _adjacents: adjacents,
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
    use crate::{query_parsing_mk2::normalisation::normalise, TypeIndex};

    #[test]
    fn extracts_inline_input_types() {
        let schema = load_schema();
        let type_index = Rc::new(TypeIndex::from_schema(&schema));
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
        let input_objects = extract_input_objects(&normalised).unwrap();

        assert_eq!(input_objects.len(), 2);
    }

    #[test]
    fn deduplicates_input_types_if_same() {
        let schema = load_schema();
        let type_index = Rc::new(TypeIndex::from_schema(&schema));
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
        let input_objects = extract_input_objects(&normalised).unwrap();

        assert_eq!(input_objects.len(), 1);
    }

    #[test]
    fn finds_variable_input_types() {
        let schema = load_schema();
        let type_index = Rc::new(TypeIndex::from_schema(&schema));
        let query = graphql_parser::parse_query::<&str>(
            r#"
              query MyQuery($input: IssueFilters!) {
                cynic: repository(owner: "obmarg", name: "cynic") {
                  issues(filterBy: $input) {
                    nodes {
                      title
                    }
                  }
                }
              	kazan: repository(owner: "obmarg", name: "kazan") {
                  issues(filterBy: $input) {
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
        let input_objects = extract_input_objects(&normalised).unwrap();

        assert_eq!(input_objects.len(), 1);
    }

    fn load_schema() -> schema::Document<'static> {
        graphql_parser::parse_schema::<&str>(include_str!("../../tests/schemas/github.graphql"))
            .unwrap()
    }
}
