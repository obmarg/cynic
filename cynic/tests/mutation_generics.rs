#[allow(non_snake_case, non_camel_case_types)]
mod schema {
    cynic::use_schema!("../schemas/raindancer.graphql");
}

#[derive(cynic::QueryVariables, Debug)]
pub struct SignInVariables<'a> {
    pub input: SignInInput<'a>,
}

#[derive(cynic::InputObject, Debug)]
#[cynic(schema_path = "../schemas/raindancer.graphql")]
pub struct SignInInput<'a> {
    pub password: &'a str,
    pub username: String,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    graphql_type = "MutationRoot",
    variables = "SignInVariables",
    schema_path = "../schemas/raindancer.graphql"
)]
pub struct SignIn {
    #[arguments(input: $input)]
    pub sign_in: String,
}

#[test]
fn test_query_building() {
    use cynic::MutationBuilder;

    let operation = SignIn::build(SignInVariables {
        input: SignInInput {
            password: "password?",
            username: "username".to_owned(),
        },
    });

    insta::assert_snapshot!(operation.query);
}
