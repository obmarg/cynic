use std::borrow::Cow;

#[allow(non_snake_case, non_camel_case_types)]
mod schema {
    cynic::use_schema!("../schemas/raindancer.graphql");
}

#[derive(cynic::QueryVariables, Debug)]
pub struct SignInVariables<'a> {
    pub input: SignInInput<'a, Cow<'a, str>>,
}

#[derive(cynic::QueryVariables, Debug)]
pub struct SignInVariablesMoreGeneric<'a, Username: cynic::schema::IsScalar<String>> {
    #[cynic(graphql_type = "SignInInput")]
    pub input: SignInInput<'a, Username>,
}

#[derive(cynic::InputObject, Debug)]
#[cynic(schema_path = "../schemas/raindancer.graphql")]
pub struct SignInInput<'a, Username: cynic::schema::IsScalar<String>> {
    pub password: &'a str,
    pub username: Username,
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

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    graphql_type = "MutationRoot",
    variables = "SignInVariablesMoreGeneric",
    schema_path = "../schemas/raindancer.graphql"
)]
pub struct SignInMoreGeneric<SI>
where
    for<'de2> SI: cynic::schema::IsOutputScalar<'de2, String>,
    for<'de2> <SI as cynic::schema::IsOutputScalar<'de2, String>>::SchemaType:
        cynic::queries::IsFieldType<String>,
{
    #[arguments(input: $input)]
    pub sign_in: SI,
}

#[test]
fn test_query_building() {
    use cynic::MutationBuilder;

    let operation = SignIn::build(SignInVariables {
        input: SignInInput {
            password: "password?",
            username: Cow::Borrowed("username"),
        },
    });

    insta::assert_snapshot!(operation.query);
}

#[test]
fn test_query_building_more_generic() {
    use cynic::MutationBuilder;

    let operation = SignInMoreGeneric::<Cow<'static, str>>::build(SignInVariablesMoreGeneric {
        input: SignInInput {
            password: "password!",
            username: &&&"username",
        },
    });

    insta::assert_snapshot!(operation.query);
}
