use std::{
    collections::{BTreeSet, HashMap},
    rc::Rc,
};

use super::{
    normalisation::{NormalisedDocument, Selection, SelectionSet},
    sorting::Vertex,
};
use crate::{
    schema::{self, InputType, InputTypeRef},
    Error,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InputObject<'schema> {
    pub schema_type: schema::InputObjectDetails<'schema>,
    pub fields: Vec<schema::InputField<'schema>>,
    // Named _adjacents so as not to clash with the adjacents func in the Vertex trait
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
        extract_objects_from_selection_set(selection_set, &mut result)?;
    }

    // Find any query variables that are input objects
    for operation in &doc.operations {
        for variable in &operation.variables {
            let variable_type = variable.value_type.inner_ref().lookup()?;

            if let InputType::InputObject(input_obj) = variable_type {
                extract_whole_input_object_tree(&input_obj, &mut result)?;
            }
        }
    }

    Ok(result)
}

fn extract_objects_from_selection_set<'query, 'schema>(
    selection_set: &SelectionSet<'query, 'schema>,
    input_objects: &mut InputObjectSet<'schema>,
) -> Result<(), Error> {
    if selection_set.selections.is_empty() {
        return Ok(());
    }

    for selection in &selection_set.selections {
        let Selection::Field(field) = selection;
        for selection_set in field.field.selection_sets() {
            extract_objects_from_selection_set(selection_set.as_ref(), input_objects)?;
        }
    }

    Ok(())
}

pub fn extract_whole_input_object_tree<'schema>(
    input_object: &schema::InputObjectDetails<'schema>,
    input_objects: &mut InputObjectSet<'schema>,
) -> Result<Rc<InputObject<'schema>>, Error> {
    let mut object_map = HashMap::new();

    let rv = extract_whole_input_object(input_object, &mut object_map)?;

    input_objects.extend(object_map.into_iter().map(|(_, obj)| obj));

    Ok(rv)
}

fn extract_whole_input_object<'schema>(
    input_object: &schema::InputObjectDetails<'schema>,
    input_objects: &mut HashMap<schema::InputObjectDetails<'schema>, Rc<InputObject<'schema>>>,
) -> Result<Rc<InputObject<'schema>>, Error> {
    let mut fields = Vec::new();
    let mut adjacents = Vec::new();

    if let Some(existing_obj) = input_objects.get(input_object) {
        return Ok(Rc::clone(existing_obj));
    }

    for field in &input_object.fields {
        let field_type = field.value_type.inner_ref().lookup()?;

        if let InputType::InputObject(inner_obj) = field_type {
            adjacents.push(extract_whole_input_object(&inner_obj, input_objects)?);
        }

        fields.push(field.clone());
    }

    let rv = Rc::new(InputObject {
        schema_type: input_object.clone(),
        _adjacents: adjacents,
        fields,
    });

    input_objects.insert(input_object.clone(), Rc::clone(&rv));

    Ok(rv)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{query_parsing::normalisation::normalise, TypeIndex};

    #[test]
    fn deduplicates_input_types_if_same() {
        let schema = load_schema();
        let type_index = Rc::new(TypeIndex::from_schema(&schema));
        let query = graphql_parser::parse_query::<&str>(
            r#"
              query ($filterOne: IssueFilters!, $filterTwo: IssueFilters!) {
                cynic: repository(owner: "obmarg", name: "cynic") {
                  issues(filterBy: $filterOne) {
                    nodes {
                      title
                    }
                  }
                }
              	kazan: repository(owner: "obmarg", name: "kazan") {
                  issues(filterBy: $filterTwo) {
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
        graphql_parser::parse_schema::<&str>(include_str!("../../../schemas/github.graphql"))
            .unwrap()
    }
}
