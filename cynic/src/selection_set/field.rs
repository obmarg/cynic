use crate::Argument;

pub enum OperationType {
    Query,
    Mutation,
}

pub enum Field {
    Root(Vec<Field>, OperationType),
    Leaf(String, Vec<Argument>),
    Composite(String, Vec<Argument>, Vec<Field>),
    InlineFragment(String, Vec<Field>),
}

impl Field {
    pub fn query<'a>(
        self,
        indent: usize,
        indent_size: usize,
        arguments_out: &mut Vec<Argument>,
    ) -> String {
        match self {
            Field::Leaf(field_name, args) => {
                let arguments = handle_field_arguments(args, arguments_out);
                format!(
                    "{:indent$}{field_name}{arguments}\n",
                    "",
                    field_name = field_name,
                    arguments = arguments,
                    indent = indent
                )
            }
            Field::Composite(field_name, args, child_fields) => {
                let arguments = handle_field_arguments(args, arguments_out);
                let child_query: String = child_fields
                    .into_iter()
                    .map(|f| f.query(indent + indent_size, indent_size, arguments_out))
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
            Field::InlineFragment(type_name, child_fields) => {
                let child_query: String = child_fields
                    .into_iter()
                    .map(|f| f.query(indent + indent_size, indent_size, arguments_out))
                    .collect();

                format!(
                    "{0:indent$}... on {type_name} {{\n{child_query}{0:indent$}}}\n",
                    "",
                    type_name = type_name,
                    child_query = child_query,
                    indent = indent
                )
            }
            Field::Root(fields, operation_type) => {
                let child_query: String = fields
                    .into_iter()
                    .map(|f| f.query(indent + indent_size, indent_size, arguments_out))
                    .collect();

                let arguments = handle_query_arguments(arguments_out);

                let operation_def = match operation_type {
                    OperationType::Query => "query Query",
                    OperationType::Mutation => "mutation Mutation",
                };

                format!(
                    "{operation_def}{arguments} {{\n{child_query}}}\n",
                    operation_def = operation_def,
                    arguments = arguments,
                    child_query = child_query
                )
            }
        }
    }
}

/// Extracts any argument values & returns a string to be used in a query.
fn handle_field_arguments<'a>(
    arguments: Vec<Argument>,
    arguments_out: &mut Vec<Argument>,
) -> String {
    if arguments.is_empty() {
        "".to_string()
    } else {
        let mut argument_index = arguments_out.len();

        let comma_seperated = arguments
            .into_iter()
            .map(|arg| {
                let rv = format!("{}: $_{}", arg.name, argument_index);
                arguments_out.push(arg);
                argument_index += 1;
                rv
            })
            .collect::<Vec<_>>()
            .join(", ");

        format!("({})", comma_seperated)
    }
}

/// Extracts any argument values & returns a string to be used in a query.
fn handle_query_arguments<'a>(arguments: &'a Vec<Argument>) -> String {
    if arguments.is_empty() {
        "".to_string()
    } else {
        let comma_seperated = arguments
            .iter()
            .enumerate()
            .map(|(i, arg)| format!("$_{}: {}", i, arg.type_))
            .collect::<Vec<_>>()
            .join(", ");

        format!("({})", comma_seperated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
    fn test_inline_fragments() {
        let fields = Field::Composite(
            "test".to_string(),
            vec![],
            vec![
                Field::Leaf("__typename".to_string(), vec![]),
                Field::InlineFragment(
                    "TypeOne".to_string(),
                    vec![Field::Leaf("a_field".to_string(), vec![])],
                ),
                Field::InlineFragment(
                    "TypeTwo".to_string(),
                    vec![Field::Leaf("another_field".to_string(), vec![])],
                ),
            ],
        );
        let mut arguments = vec![];
        assert_eq!(
            fields.query(0, 2, &mut arguments),
            "test {\n  __typename\n  ... on TypeOne {\n    a_field\n  }\n  ... on TypeTwo {\n    another_field\n  }\n}\n"
        );
    }

    #[test]
    fn test_query_with_arguments() {
        let fields = Field::Composite(
            "test_struct".to_string(),
            vec![Argument::new("an_arg", "Bool!", false)],
            vec![
                Field::Leaf("field_one".to_string(), vec![]),
                Field::Composite(
                    "nested".to_string(),
                    vec![],
                    vec![Field::Leaf(
                        "a_string".to_string(),
                        vec![Argument::new("another_arg", "Bool!", true)],
                    )],
                ),
            ],
        );
        let mut arguments = vec![];

        assert_eq!(
            fields.query(0, 2, &mut arguments),
            "test_struct(an_arg: $_0) {\n  field_one\n  nested {\n    a_string(another_arg: $_1)\n  }\n}\n"
        );
        assert_eq!(
            arguments
                .iter()
                .map(|a| a.serialize_result.as_ref().unwrap())
                .collect::<Vec<_>>(),
            vec![&json!(false), &json!(true)]
        );
        assert_eq!(
            arguments
                .iter()
                .map(|a| a.type_.clone())
                .collect::<Vec<_>>(),
            vec!["Bool!", "Bool!"]
        );
    }
}
