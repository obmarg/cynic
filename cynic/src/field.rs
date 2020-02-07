use crate::Argument;

pub enum Field {
    Root(Vec<Field>),
    Leaf(String, Vec<Argument>),
    Composite(String, Vec<Argument>, Vec<Field>),
}

impl Field {
    pub(crate) fn query<'a>(
        &'a self,
        indent: usize,
        indent_size: usize,
        argument_values: &mut Vec<Result<&'a serde_json::Value, ()>>,
        argument_types: &mut Vec<&'a str>,
    ) -> String {
        match self {
            Field::Leaf(field_name, args) => {
                let arguments = handle_field_arguments(args, argument_values, argument_types);
                format!(
                    "{:indent$}{field_name}{arguments}\n",
                    "",
                    field_name = field_name,
                    arguments = arguments,
                    indent = indent
                )
            }
            Field::Composite(field_name, args, child_fields) => {
                let arguments = handle_field_arguments(args, argument_values, argument_types);
                let child_query: String = child_fields
                    .iter()
                    .map(|f| {
                        f.query(
                            indent + indent_size,
                            indent_size,
                            argument_values,
                            argument_types,
                        )
                    })
                    .collect();

                format!(
                    "{0:indent$}{field_name}{arguments} {{\n{child_query}{0:indent$}}}\n",
                    "",
                    field_name = field_name,
                    child_query = child_query,
                    indent = indent,
                    arguments = arguments
                )
            }
            Field::Root(fields) => {
                let child_query: String = fields
                    .iter()
                    .map(|f| {
                        f.query(
                            indent + indent_size,
                            indent_size,
                            argument_values,
                            argument_types,
                        )
                    })
                    .collect();

                let arguments = handle_query_arguments(argument_types);

                format!(
                    "query Query{arguments} {{\n{child_query}}}\n",
                    arguments = arguments,
                    child_query = child_query
                )
            }
        }
    }
}

/// Extracts any argument values & returns a string to be used in a query.
fn handle_field_arguments<'a>(
    arguments: &'a Vec<Argument>,
    argument_values: &mut Vec<Result<&'a serde_json::Value, ()>>,
    argument_types: &mut Vec<&'a str>,
) -> String {
    if arguments.is_empty() {
        "".to_string()
    } else {
        let mut argument_index = argument_values.len();

        let comma_seperated = arguments
            .iter()
            .map(|arg| {
                argument_values.push(arg.value.as_ref().map_err(|_| ()).clone());
                argument_types.push(&arg.type_);
                let rv = format!("{}: $_{}", arg.name, argument_index);
                argument_index += 1;
                rv
            })
            .collect::<Vec<_>>()
            .join(", ");

        format!("({})", comma_seperated)
    }
}

/// Extracts any argument values & returns a string to be used in a query.
fn handle_query_arguments<'a>(argument_types: &'a Vec<&str>) -> String {
    if argument_types.is_empty() {
        "".to_string()
    } else {
        let comma_seperated = argument_types
            .iter()
            .enumerate()
            .map(|(i, arg_type)| format!("$_{}: {}", i, arg_type))
            .collect::<Vec<_>>()
            .join(", ");

        format!("({})", comma_seperated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_building() {
        let fields = Field::Composite(
            "test_struct".to_string(),
            vec![],
            vec![
                Field::Leaf("field_one".to_string(), vec![]),
                Field::Composite(
                    "nested".to_string(),
                    vec![],
                    vec![Field::Leaf("a_string".to_string(), vec![])],
                ),
            ],
        );
        let mut arguments = vec![];
        let mut argument_types = vec![];

        assert_eq!(
            fields.query(0, 2, &mut arguments, &mut argument_types),
            "test_struct {\n  field_one\n  nested {\n    a_string\n  }\n}\n"
        );
        assert!(arguments.is_empty());
        assert!(argument_types.is_empty());
    }

    #[test]
    fn test_query_with_arguments() {
        let fields = Field::Composite(
            "test_struct".to_string(),
            vec![Argument::new(
                "an_arg",
                "Bool!",
                serde_json::Value::Bool(false),
            )],
            vec![
                Field::Leaf("field_one".to_string(), vec![]),
                Field::Composite(
                    "nested".to_string(),
                    vec![],
                    vec![Field::Leaf(
                        "a_string".to_string(),
                        vec![Argument::new(
                            "another_arg",
                            "Bool!",
                            serde_json::Value::Bool(true),
                        )],
                    )],
                ),
            ],
        );
        let mut arguments = vec![];
        let mut argument_types = vec![];

        assert_eq!(
            fields.query(0, 2, &mut arguments, &mut argument_types),
            "test_struct(an_arg: $_0) {\n  field_one\n  nested {\n    a_string(another_arg: $_1)\n  }\n}\n"
        );
        assert_eq!(
            arguments
                .into_iter()
                .map(|v| serde_json::from_value::<bool>(v.unwrap().clone()).unwrap())
                .collect::<Vec<_>>(),
            vec![false, true]
        );
        assert_eq!(argument_types, vec!["Bool!", "Bool!"]);
    }
}
