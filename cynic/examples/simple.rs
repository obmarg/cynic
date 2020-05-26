fn main() {
    //println!("{}", cynic::to_query::<TestStruct>(TestArgs {}));
}

mod query_dsl {
    type Json = serde_json::Value;

    cynic::query_dsl!("cynic/examples/simple.graphql");
}

use cynic::selection_set;

#[derive(Clone, cynic::FragmentArguments)]
struct TestArgs {}

#[derive(cynic::QueryFragment)]
#[cynic(
    schema_path = "cynic/examples/simple.graphql",
    query_module = "query_dsl",
    graphql_type = "TestStruct",
    argument_struct = "TestArgs"
)]
struct TestStruct {
    #[cynic_arguments(x = Some(1), y = Some("1".to_string()))]
    field_one: String,
    nested: Nested,
    opt_nested: Option<Nested>,
}

#[derive(cynic::QueryFragment)]
#[cynic(
    schema_path = "cynic/examples/simple.graphql",
    query_module = "query_dsl",
    graphql_type = "Nested"
)]
struct Nested {
    a_string: String,
    opt_string: Option<String>,
}

#[derive(cynic::QueryFragment)]
#[cynic(
    schema_path = "cynic/examples/simple.graphql",
    query_module = "query_dsl",
    graphql_type = "TestStruct"
)]
struct Test {
    #[cynic_arguments(x = Some(1), y = Some("1".to_string()))]
    field_one: String,
}

impl Test {
    fn new(field_one: String) -> Self {
        Test { field_one }
    }
}

#[derive(cynic::InlineFragments)]
#[cynic(
    schema_path = "cynic/examples/simple.graphql",
    query_module = "query_dsl",
    graphql_type = "MyUnionType"
)]
enum MyUnionType {
    Test(Test),
    Nested(Nested),
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

    fn query() -> Self::SelectionSet {
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

    fn query() -> Self::SelectionSet {
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
        fn encode(&self) -> Result<serde_json::Value, ()> {
            todo!()
        }
    }

    // Another custom scalar
    struct Upload;

    //cynic::query_dsl!("cms-schema.gql");
}
