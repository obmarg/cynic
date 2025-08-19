use cynic::MutationBuilder;
use std::borrow::Cow;

#[allow(non_snake_case, non_camel_case_types)]
mod raindancer {
    cynic::use_schema!("../schemas/raindancer.graphql");
}

#[derive(cynic::InputObject, Debug)]
#[cynic(
    schema_path = "../schemas/raindancer.graphql",
    schema_module = "raindancer"
)]
pub struct SignInInput<'a, Username: cynic::schema::IsScalar<String>> {
    pub password: &'a str,
    pub username: Username,
}

#[test]
fn test_query_building() {
    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(
        graphql_type = "MutationRoot",
        variables = "SignInVariables",
        schema_path = "../schemas/raindancer.graphql",
        schema_module = "raindancer"
    )]
    pub struct SignIn {
        #[arguments(input: $input)]
        #[allow(dead_code)]
        pub sign_in: String,
    }

    #[derive(cynic::QueryVariables, Debug)]
    #[cynic(schema_module = "raindancer")]
    pub struct SignInVariables<'a> {
        pub input: SignInInput<'a, Cow<'a, str>>,
    }

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

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(
        graphql_type = "MutationRoot",
        variables = "SignInVariablesMoreGeneric",
        schema_path = "../schemas/raindancer.graphql",
        schema_module = "raindancer"
    )]
    pub struct SignInMoreGeneric<SI: cynic::schema::IsScalar<String>>
    where
        <SI as cynic::schema::IsScalar<String>>::SchemaType: cynic::queries::IsFieldType<String>,
    {
        #[arguments(input: $input)]
        #[allow(dead_code)]
        pub sign_in: SI,
    }
    #[derive(cynic::QueryVariables, Debug)]
    #[cynic(schema_module = "raindancer")]
    pub struct SignInVariablesMoreGeneric<'a, Username: cynic::schema::IsScalar<String>> {
        #[cynic(graphql_type = "SignInInput")]
        pub input: SignInInput<'a, Username>,
    }

    let operation = SignInMoreGeneric::<Cow<'static, str>>::build(SignInVariablesMoreGeneric {
        input: SignInInput {
            password: "password!",
            username: &&&"username",
        },
    });

    insta::assert_snapshot!(operation.query);
}

#[allow(non_snake_case, non_camel_case_types)]
mod test_cases {
    cynic::use_schema!("../schemas/test_cases.graphql");
}

#[test]
fn test_query_building_nested_generic_in_vec() {
    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(
        graphql_type = "MutationRoot",
        variables = "SendManyBazVariables",
        schema_path = "../schemas/test_cases.graphql",
        schema_module = "test_cases"
    )]
    pub struct SendManyBaz {
        #[arguments(many_baz: $many_baz)]
        #[allow(dead_code)]
        pub send_many_baz: Option<i32>,
    }

    #[derive(cynic::QueryVariables, Debug)]
    #[cynic(schema_module = "test_cases")]
    pub struct SendManyBazVariables<'a, Id: cynic::schema::IsScalar<cynic::Id>> {
        #[cynic(graphql_type = "Vec<test_cases::Baz>")]
        pub many_baz: Vec<Baz<'a, Id>>,
    }

    #[derive(cynic::InputObject, Debug)]
    #[cynic(
        schema_path = "../schemas/test_cases.graphql",
        schema_module = "test_cases"
    )]
    pub struct Baz<'a, Id: cynic::schema::IsScalar<cynic::Id>> {
        pub id: Id,
        pub a_string: &'a str,
    }

    let operation = SendManyBaz::build(SendManyBazVariables {
        many_baz: vec![Baz {
            id: cynic::Id::new("some-totally-correct-id"),
            a_string: "baz",
        }],
    });

    insta::assert_snapshot!(operation.query);
}

#[test]
fn test_with_optional_string_type() {
    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(
        graphql_type = "MutationRoot",
        variables = "SendManyBazVariables",
        schema_path = "../schemas/test_cases.graphql",
        schema_module = "test_cases"
    )]
    pub struct SendManyBaz {
        #[arguments(many_baz: $many_baz)]
        #[allow(dead_code)]
        pub send_many_baz: Option<i32>,
    }

    #[derive(cynic::QueryVariables, Debug)]
    #[cynic(schema_module = "test_cases")]
    pub struct SendManyBazVariables<'a, Id: cynic::schema::IsScalar<cynic::Id>> {
        #[cynic(graphql_type = "Vec<test_cases::Baz>")]
        pub many_baz: Vec<Baz<'a, Id>>,
    }

    #[derive(cynic::InputObject, Debug)]
    #[cynic(
        schema_path = "../schemas/test_cases.graphql",
        schema_module = "test_cases"
    )]
    pub struct Baz<'a, Id: cynic::schema::IsScalar<cynic::Id>> {
        pub id: Id,
        pub a_string: &'a str,
        pub an_optional_string: &'a str,
    }

    let operation = SendManyBaz::build(SendManyBazVariables {
        many_baz: vec![Baz {
            id: cynic::Id::new("some-totally-correct-id"),
            a_string: "baz",
            an_optional_string: "baz",
        }],
    });

    insta::assert_snapshot!(operation.query, @r"
    mutation SendManyBaz($manyBaz: [Baz!]!) {
      sendManyBaz(many_baz: $manyBaz)
    }
    ");
}
