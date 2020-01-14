fn main() {
    println!("{}", cynic::to_query::<TestStruct>(()));
}

mod query_dsl {
    type JSON = serde_json::Value;

    cynic::query_dsl!("cynic/examples/simple.graphql");
}

use cynic::selection_set;

#[derive(cynic::QueryFragment)]
#[cynic(
    schema_path = "cynic/examples/simple.graphql",
    query_module = "query_dsl",
    graphql_type = "TestStruct"
)]
struct TestStruct {
    field_one: String,
    nested: Nested,
}

#[derive(cynic::QueryFragment)]
#[cynic(
    schema_path = "cynic/examples/simple.graphql",
    query_module = "query_dsl",
    graphql_type = "Nested"
)]
struct Nested {
    a_string: String,
}

#[derive(cynic::QueryFragment)]
#[cynic(
    schema_path = "cynic/examples/simple.graphql",
    query_module = "query_dsl",
    graphql_type = "TestStruct",
    // Argument type can default to () or something
    // argument_type = "Args"
)]
struct Test {
    #[cynic_arguments(x = args.xyz, y = "1")]
    field_one: String,
}

impl Test {
    fn new(field_one: String) -> Self {
        Test { field_one }
    }
}

/*
fn query() {
    let query = query_dsl::Query::test_struct(selection_set::map2(
        TestStruct::new,
        query_dsl::TestStruct::field_one(),
        query_dsl::TestStruct::nested(selection_set::map(
            Nested::new,
            query_dsl::Nested::a_string(),
        )),
    ));
}*/

impl cynic::QueryRoot for query_dsl::TestStruct {}

// TODO: Some sort of ToQuery trait
// That's only implemented when QueryFragment::SelectionSet::TypeLock == RootQuery
// TODO: I should figure out how arguments could work?

/*

impl cynic::QueryFragment<'static> for TestStruct {
    type SelectionSet = selection_set::SelectionSet<'static, Self, query_dsl::TestStruct>;
    type Arguments = ArgStruct;

    fn selection_set() -> Self::SelectionSet {
        // TODO: Got to say I'm not that enamoured with this syntax.
        // Is there a better way to write this?
        selection_set::map2(
            TestStruct::new,
            query_dsl::TestStruct::field_one(),
            query_dsl::TestStruct::nested(Nested::selection_set()),
        )
    }
}

impl cynic::QueryFragment<'static> for Nested {
    type SelectionSet = selection_set::SelectionSet<'static, Self, query_dsl::Nested>;

    fn selection_set() -> Self::SelectionSet {
        selection_set::map(Nested::new, query_dsl::Nested::a_string())
    }
}
*/

mod test {

    type JSON = serde_json::Value;

    // A custom scalars.
    pub struct DateTime {}

    impl cynic::Scalar for DateTime {
        fn decode(_: &serde_json::Value) -> Result<Self, json_decode::DecodeError> {
            Ok(DateTime {})
        }
    }

    // Another custom scalar
    struct Upload;

    cynic::query_dsl!("cms-schema.gql");
}
