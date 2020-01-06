use crate::Argument;

pub enum Field {
    Leaf(String, Vec<Argument>),
    Composite(String, Vec<Argument>, Vec<Field>),
}

impl Field {
    pub(crate) fn query<'a>(
        &'a self,
        indent: usize,
        indent_size: usize,
        argument_values: &mut Vec<&'a serde_json::Value>,
    ) -> String {
        match self {
            Field::Leaf(field_name, args) => {
                let arguments = handle_arguments(args, argument_values);
                format!(
                    "{:indent$}{field_name}{arguments}\n",
                    "",
                    field_name = field_name,
                    arguments = arguments,
                    indent = indent
                )
            }
            Field::Composite(field_name, args, child_fields) => {
                let arguments = handle_arguments(args, argument_values);
                let child_query: String = child_fields
                    .iter()
                    .map(|f| f.query(indent + indent_size, indent_size, argument_values))
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
        }
    }
}

/// Extracts any argument values & returns a string to be used in a query.
fn handle_arguments<'a>(
    arguments: &'a Vec<Argument>,
    argument_values: &mut Vec<&'a serde_json::Value>,
) -> String {
    if arguments.is_empty() {
        "".to_string()
    } else {
        let mut argument_index = argument_values.len();

        let comma_seperated = arguments
            .iter()
            .map(|arg| {
                argument_values.push(&arg.value);
                let rv = format!("{}=${}", arg.name, argument_index);
                argument_index += 1;
                rv
            })
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

        assert_eq!(
            fields.query(0, 2, &mut arguments),
            "test_struct {\n  field_one\n  nested {\n    a_string\n  }\n}\n"
        );
        assert!(arguments.is_empty());
    }

    #[test]
    fn test_query_with_arguments() {
        let fields = Field::Composite(
            "test_struct".to_string(),
            vec![Argument::new("an_arg", serde_json::Value::Bool(false))],
            vec![
                Field::Leaf("field_one".to_string(), vec![]),
                Field::Composite(
                    "nested".to_string(),
                    vec![],
                    vec![Field::Leaf(
                        "a_string".to_string(),
                        vec![Argument::new("another_arg", serde_json::Value::Bool(true))],
                    )],
                ),
            ],
        );
        let mut arguments = vec![];

        assert_eq!(
            fields.query(0, 2, &mut arguments),
            "test_struct(an_arg=$0) {\n  field_one\n  nested {\n    a_string(another_arg=$1)\n  }\n}\n"
        );
        assert_eq!(
            arguments
                .into_iter()
                .map(|v| serde_json::from_value::<bool>(v.clone()).unwrap())
                .collect::<Vec<_>>(),
            vec![false, true]
        )
    }
}
