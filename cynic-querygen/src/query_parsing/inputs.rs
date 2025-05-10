use std::{
    collections::{BTreeSet, HashMap, HashSet},
    rc::Rc,
};

use {
    super::{normalisation::NormalisedDocument, sorting::Vertex},
    crate::{
        output::InputObjectField,
        schema::{self, InputType, InputTypeRef},
        Error,
    },
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InputObject<'schema> {
    pub schema_type: schema::InputObjectDetails<'schema>,
    pub fields: Vec<InputObjectField<'schema>>,
    // Named children_ so as not to clash with the children func in the Vertex trait
    children_: Vec<Rc<InputObject<'schema>>>,
    needs_lifetime_a: bool,
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
    fn children(self: &Rc<InputObject<'schema>>) -> Vec<Rc<InputObject<'schema>>> {
        self.children_.clone()
    }
}

pub type InputObjectSet<'schema> = BTreeSet<Rc<InputObject<'schema>>>;

pub fn extract_input_objects<'schema>(
    doc: &NormalisedDocument<'_, 'schema>,
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

    input_objects.extend(object_map.into_values());

    Ok(rv)
}

fn extract_whole_input_object<'schema>(
    input_object: &schema::InputObjectDetails<'schema>,
    input_objects: &mut HashMap<schema::InputObjectDetails<'schema>, Rc<InputObject<'schema>>>,
    seen_objects: &mut HashSet<schema::InputObjectDetails<'schema>>,
) -> Result<Rc<InputObject<'schema>>, Error> {
    let mut fields = Vec::new();
    let mut children = Vec::new();
    let mut push_child = |child: Rc<InputObject<'schema>>| -> bool {
        let this_one_needs_lifetime_a = child.needs_lifetime_a;
        children.push(child);
        this_one_needs_lifetime_a
    };
    let mut needs_lifetime_a_for_current_object = false;

    seen_objects.insert(input_object.clone());

    for field in &input_object.fields {
        let field_type_as_inputtype = field.value_type.inner_ref().lookup()?;
        let mut needs_boxed_field = false;

        let is_sub_object_with_lifetime_for_field_type =
            if let InputType::InputObject(inner_obj_details) = field_type_as_inputtype {
                if let Some(existing_obj_rc) = input_objects.get(&inner_obj_details) {
                    push_child(Rc::clone(existing_obj_rc))
                } else if seen_objects.contains(&inner_obj_details) {
                    needs_boxed_field = true;
                    // Enhanced heuristic for recursive types:
                    let mut has_lifetime_source = false;
                    for f_of_recursive_obj in &inner_obj_details.fields {
                        // Iterate fields of the recursive type definition
                        let type_of_f = f_of_recursive_obj.value_type.inner_ref().lookup()?;
                        match type_of_f {
                            InputType::Scalar(s) if s.name == "String" || s.name == "ID" => {
                                has_lifetime_source = true;
                                break;
                            }
                            InputType::InputObject(nested_details) => {
                                // Check if this nested input object (that is part of the recursive type's definition)
                                // itself implies a lifetime.
                                // Avoid infinite recursion if nested_details is the same as inner_obj_details for this specific check path.
                                if nested_details.name != inner_obj_details.name {
                                    if let Some(processed_nested_obj) =
                                        input_objects.get(&nested_details)
                                    {
                                        if processed_nested_obj.needs_lifetime_a {
                                            has_lifetime_source = true;
                                            break;
                                        }
                                    } else {
                                        // Not yet fully processed (and not on current stack implies it would be a new branch if explored from here).
                                        // Perform a shallow check on its fields for immediate lifetime sources.
                                        for f_of_nested in &nested_details.fields {
                                            if let InputType::Scalar(s_nested) =
                                                f_of_nested.value_type.inner_ref().lookup()?
                                            {
                                                if s_nested.name == "String"
                                                    || s_nested.name == "ID"
                                                {
                                                    has_lifetime_source = true;
                                                    break; // Breaks inner loop (f_of_nested)
                                                }
                                            }
                                        }
                                        if has_lifetime_source {
                                            break;
                                        } // Breaks outer loop (f_of_recursive_obj)
                                    }
                                }
                            }
                            _ => {} // Other scalar types, Enums don't confer lifetimes this way
                        }
                    }
                    has_lifetime_source
                } else {
                    push_child(extract_whole_input_object(
                        &inner_obj_details,
                        input_objects,
                        seen_objects,
                    )?)
                }
            } else {
                false
            };

        let type_spec = field.value_type.type_spec(
            needs_boxed_field,
            false,
            is_sub_object_with_lifetime_for_field_type,
        );

        if type_spec.contains_lifetime_a {
            needs_lifetime_a_for_current_object = true;
        }

        fields.push(InputObjectField {
            type_spec,
            schema_field: field.clone(),
        });
    }

    let rv = Rc::new(InputObject {
        schema_type: input_object.clone(),
        children_: children,
        fields,
        needs_lifetime_a: needs_lifetime_a_for_current_object,
    });

    input_objects.insert(input_object.clone(), Rc::clone(&rv));
    seen_objects.remove(input_object);

    Ok(rv)
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::{query_parsing::normalisation::normalise, TypeIndex},
        cynic_parser::{type_system::ids::FieldDefinitionId, TypeSystemDocument},
        schema::add_builtins,
        std::sync::LazyLock,
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
        let input_objects = extract_input_objects(&normalised).unwrap();

        assert_eq!(input_objects.len(), 1);
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
        let input_objects = extract_input_objects(&normalised).unwrap();

        assert_eq!(input_objects.len(), 1);
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
        let input_objects = extract_input_objects(&normalised).unwrap();

        assert_eq!(input_objects.len(), 3);
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
