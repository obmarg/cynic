use std::collections::HashSet;

use crate::schema::{InputFieldType, InputObjectDetails};

use super::Variable;

use {
    super::normalisation::NormalisedDocument,
    crate::schema::{InputType, InputTypeRef},
};

impl<'schema> InputObjectDetails<'schema> {
    /// Extracts any input types used by this InputObject
    pub fn required_input_types(&self) -> Vec<InputTypeRef<'schema>> {
        self.fields
            .iter()
            .map(|field| field.value_type.inner_ref().clone())
            .collect()
    }
}

pub struct InputObjects<'a> {
    objects: Vec<InputObjectDetails<'a>>,
    recursive_objects: HashSet<&'a str>,

    objects_with_lifetime: HashSet<&'a str>,
}

impl<'a> InputObjects<'a> {
    pub fn new(document: &NormalisedDocument<'a, 'a>) -> Self {
        let objects = InputObjectIter::from_variables(
            document
                .operations
                .iter()
                .flat_map(|operation| operation.variables.iter().cloned()),
        )
        .collect();
        let recursive_objects = recursive_objects(document);
        let objects_with_lifetime = lifetimed_objects(document);

        InputObjects {
            objects,
            recursive_objects,
            objects_with_lifetime,
        }
    }

    pub fn required_input_types(&self) -> impl Iterator<Item = InputTypeRef<'a>> + '_ {
        self.objects
            .iter()
            .flat_map(|object| object.required_input_types())
    }

    pub fn processed_objects(&self) -> Vec<crate::output::InputObject<'a>> {
        self.objects
            .iter()
            .map(|object| crate::output::InputObject {
                name: object.name.to_string(),
                fields: object
                    .fields
                    .iter()
                    .map(|field| {
                        let inner_type = field.value_type.inner_name();
                        let inner_type = inner_type.as_ref();
                        let needs_boxed = self.recursive_objects.contains(inner_type);
                        let requires_lifetime = self.objects_with_lifetime.contains(inner_type);
                        crate::output::InputObjectField {
                            schema_field: field.clone(),
                            type_spec: if object.is_oneof {
                                field
                                    .value_type
                                    .oneof_type_spec(needs_boxed, requires_lifetime)
                            } else {
                                field.value_type.type_spec(needs_boxed, requires_lifetime)
                            },
                        }
                    })
                    .collect(),
                schema_name: None,
                is_oneof: object.is_oneof,
            })
            .collect()
    }
}

fn recursive_objects<'a>(document: &NormalisedDocument<'a, 'a>) -> HashSet<&'a str> {
    let mut recursive_objects = HashSet::new();
    for document in &document.operations {
        for variable in &document.variables {
            let mut stack = vec![(variable.value_type.clone(), vec![])];
            let mut seen_objects = HashSet::new();
            while let Some((field_type, mut ancestors)) = stack.pop() {
                let Ok(InputType::InputObject(object)) = field_type.inner_ref().lookup() else {
                    continue;
                };
                if ancestors.contains(&object.name) {
                    recursive_objects.insert(object.name);
                }
                if seen_objects.contains(object.name) {
                    continue;
                }
                seen_objects.insert(object.name);
                ancestors.push(object.name);

                for field in object.fields {
                    stack.push((field.value_type, ancestors.clone()))
                }
            }
        }
    }
    recursive_objects
}

fn lifetimed_objects<'a>(document: &NormalisedDocument<'a, 'a>) -> HashSet<&'a str> {
    let mut lifetimed_objects = HashSet::new();
    for document in &document.operations {
        for variable in &document.variables {
            let mut stack = vec![variable.value_type.clone()];
            let mut visited = HashSet::new();

            'outer: while !stack.is_empty() {
                if let Ok(InputType::InputObject(object)) =
                    stack.last().unwrap().inner_ref().lookup()
                {
                    for field in &object.fields {
                        if !visited.contains(&field.value_type.inner_name()) {
                            stack.push(field.value_type.clone());
                            visited.insert(field.value_type.inner_name());
                            continue 'outer;
                        }
                    }

                    // If we get here all child field types have been seen.
                    // We need to check whether any child fields need a lifetime...
                    for field in object.fields {
                        if lifetimed_objects.contains(field.value_type.inner_name().as_ref())
                            || field.value_type.requires_lifetime()
                        {
                            lifetimed_objects.insert(object.name);
                        }
                    }
                }

                let visited_node = stack.pop().unwrap();
                visited.insert(visited_node.inner_name());
            }
        }
    }

    lifetimed_objects
}

impl InputFieldType<'_> {
    pub fn requires_lifetime(&self) -> bool {
        matches!(self.inner_name().as_ref(), "String" | "ID")
    }
}

#[derive(Clone)]
struct InputObjectIter<'a> {
    stack: Vec<InputType<'a>>,
    seen: HashSet<&'a str>,
}

impl<'a> InputObjectIter<'a> {
    fn from_variables(variables: impl IntoIterator<Item = Variable<'a, 'a>>) -> Self {
        InputObjectIter {
            stack: variables
                .into_iter()
                .filter_map(|variable| variable.value_type.inner_ref().lookup().ok())
                .collect(),
            seen: HashSet::new(),
        }
    }
}

impl<'a> Iterator for InputObjectIter<'a> {
    type Item = InputObjectDetails<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.stack.pop()? {
                InputType::Scalar(_) | InputType::Enum(_) => continue,
                InputType::InputObject(input_object_details)
                    if self.seen.contains(input_object_details.name) =>
                {
                    continue;
                }
                InputType::InputObject(input_object_details) => {
                    self.seen.insert(input_object_details.name);
                    for field in input_object_details.fields.iter().rev() {
                        self.stack
                            .extend(field.value_type.inner_ref().lookup().ok())
                    }
                    return Some(input_object_details);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{TypeIndex, add_builtins, query_parsing::normalisation::normalise},
        cynic_parser::{TypeSystemDocument, type_system::ids::FieldDefinitionId},
        std::{rc::Rc, sync::LazyLock},
    };

    #[test]
    fn deduplicates_input_types_if_same() {
        let (schema, typename_id) = &*GITHUB_SCHEMA;
        let type_index = Rc::new(TypeIndex::from_schema(schema, *typename_id));
        let query = cynic_parser::parse_executable_document(
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
        let input_objects = InputObjects::new(&normalised);

        assert_eq!(input_objects.objects.len(), 1);
    }

    #[test]
    fn finds_variable_input_types() {
        let (schema, typename_id) = &*GITHUB_SCHEMA;
        let type_index = Rc::new(TypeIndex::from_schema(schema, *typename_id));
        let query = cynic_parser::parse_executable_document(
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
        let input_objects = InputObjects::new(&normalised);

        assert_eq!(input_objects.objects.len(), 1);
    }

    #[test]
    fn test_extracting_recursive_types() {
        let (schema, typename_id) = &*TEST_CASE_SCHEMA;
        let type_index = Rc::new(TypeIndex::from_schema(schema, *typename_id));

        let query = cynic_parser::parse_executable_document(
            r#"
                query MyQuery($input: SelfRecursiveInput!, $input2: RecursiveInputParent!) {
                    recursiveInputField(recursive: $input, recursive2: $input2)
                }
            "#,
        )
        .unwrap();

        let normalised = normalise(&query, &type_index).unwrap();
        let input_objects = InputObjects::new(&normalised);

        assert_eq!(input_objects.objects.len(), 3);
    }

    static GITHUB_SCHEMA: LazyLock<(TypeSystemDocument, FieldDefinitionId)> = LazyLock::new(|| {
        let schema = cynic_parser::parse_type_system_document(include_str!(
            "../../../schemas/github.graphql"
        ))
        .unwrap();
        add_builtins(schema)
    });

    static TEST_CASE_SCHEMA: LazyLock<(TypeSystemDocument, FieldDefinitionId)> =
        LazyLock::new(|| {
            let schema = cynic_parser::parse_type_system_document(include_str!(
                "../../../schemas/test_cases.graphql"
            ))
            .unwrap();
            add_builtins(schema)
        });
}
