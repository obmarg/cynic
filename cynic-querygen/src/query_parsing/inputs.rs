use std::{
    collections::{BTreeSet, HashMap, HashSet},
    rc::Rc,
};

use super::{normalisation::NormalisedDocument, sorting::Vertex};
use crate::{
    output::InputObjectField,
    schema::{self, InputType, InputTypeRef},
    Error,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InputObject<'schema> {
    pub schema_type: schema::InputObjectDetails<'schema>,
    pub fields: Vec<InputObjectField<'schema>>,
    // Named adjacents_ so as not to clash with the adjacents func in the Vertex trait
    adjacents_: Vec<Rc<InputObject<'schema>>>,
}

impl<'schema> InputObject<'schema> {
    /// Extracts any input types used by this InputObject
    pub fn required_input_types(&self) -> Vec<InputTypeRef<'schema>> {
        self.fields
            .iter()
            .map(|field| field.schema_field.value_type.inner_ref().clone())
            .collect()
    }
}

impl<'schema> Vertex for InputObject<'schema> {
    fn adjacents(self: &Rc<InputObject<'schema>>) -> Vec<Rc<InputObject<'schema>>> {
        self.adjacents_.clone()
    }
}

pub type InputObjectSet<'schema> = BTreeSet<Rc<InputObject<'schema>>>;

pub fn extract_input_objects<'query, 'schema>(
    doc: &NormalisedDocument<'query, 'schema>,
) -> Result<InputObjectSet<'schema>, Error> {
    let mut result = InputObjectSet::new();

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

pub fn extract_whole_input_object_tree<'schema>(
    input_object: &schema::InputObjectDetails<'schema>,
    input_objects: &mut InputObjectSet<'schema>,
) -> Result<Rc<InputObject<'schema>>, Error> {
    let mut object_map = HashMap::new();
    let mut seen_objects = HashSet::new();

    let rv = extract_whole_input_object(input_object, &mut object_map, &mut seen_objects)?;

    input_objects.extend(object_map.into_iter().map(|(_, obj)| obj));

    Ok(rv)
}

fn extract_whole_input_object<'schema>(
    input_object: &schema::InputObjectDetails<'schema>,
    input_objects: &mut HashMap<schema::InputObjectDetails<'schema>, Rc<InputObject<'schema>>>,
    seen_objects: &mut HashSet<schema::InputObjectDetails<'schema>>,
) -> Result<Rc<InputObject<'schema>>, Error> {
    let mut fields = Vec::new();
    let mut adjacents = Vec::new();

    seen_objects.insert(input_object.clone());

    for field in &input_object.fields {
        let field_type = field.value_type.inner_ref().lookup()?;
        let mut needs_boxed = false;

        if let InputType::InputObject(inner_obj) = field_type {
            if let Some(existing_obj) = input_objects.get(&inner_obj) {
                adjacents.push(Rc::clone(existing_obj));
            } else if seen_objects.contains(&inner_obj) {
                // If we hit this path we've got a recursive object.
                // going to skip pushing into adjacents in that case.
                // technically it'll end up with a bad "graph" but we just use it topsort the output
                // order so it should be fine...

                // We do however need to mark this field as recursive.
                needs_boxed = true;
            } else {
                adjacents.push(extract_whole_input_object(
                    &inner_obj,
                    input_objects,
                    seen_objects,
                )?);
            }
        }

        fields.push(InputObjectField {
            schema_field: field.clone(),
            needs_boxed,
        });
    }

    let rv = Rc::new(InputObject {
        schema_type: input_object.clone(),
        adjacents_: adjacents,
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
        let schema = load_graphql_schema();
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
        let schema = load_graphql_schema();
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

    #[test]
    fn test_extracting_recursive_types() {
        let schema = load_test_schema();
        let type_index = Rc::new(TypeIndex::from_schema(&schema));

        let query = graphql_parser::parse_query::<&str>(
            r#"
                query MyQuery($input: SelfRecursiveInput!, $input2: RecursiveInputParent!) {
                    recursiveInputField(recursive: $input, recursive2: $input2)
                }
            "#,
        )
        .unwrap();

        let normalised = normalise(&query, &type_index).unwrap();
        let input_objects = extract_input_objects(&normalised).unwrap();

        assert_eq!(input_objects.len(), 3);
    }

    fn load_graphql_schema() -> schema::Document<'static> {
        graphql_parser::parse_schema::<&str>(include_str!("../../../schemas/github.graphql"))
            .unwrap()
    }

    fn load_test_schema() -> schema::Document<'static> {
        graphql_parser::parse_schema::<&str>(include_str!("../../../schemas/test_cases.graphql"))
            .unwrap()
    }
}
