fn main() {
    use queries::{TestArgs, TestStruct};

    //println!("{}", cynic::Query::new<TestStruct>(TestArgs {}));
}

mod query_dsl {
    #[derive(cynic::Scalar)]
    pub struct Json(serde_json::Value);

    cynic::query_dsl!("examples/simple.graphql");
}

use cynic::selection_set;

#[cynic::query_module(
    schema_path = "examples/simple.graphql",
    // TODO: Make query_module optional (automatically does it if missing)
    // Some complications around what to do with scalars here though...
    query_module = "query_dsl"
)]
mod queries {
    use super::query_dsl;

    #[derive(Clone, cynic::FragmentArguments)]
    pub struct TestArgs {}

    #[derive(cynic::QueryFragment)]
    #[cynic(graphql_type = "TestStruct", argument_struct = "TestArgs")]
    pub struct TestStruct {
        #[cynic_arguments(x = 1, y = "1")]
        field_one: String,
        nested: Nested,
        opt_nested: Option<Nested>,
        dessert: Option<Dessert>,
    }

    #[derive(cynic::QueryFragment)]
    #[cynic(graphql_type = "Nested")]
    pub struct Nested {
        a_string: String,
        opt_string: Option<String>,
    }

    #[derive(cynic::QueryFragment)]
    #[cynic(graphql_type = "TestStruct")]
    pub struct Test {
        #[cynic_arguments(x = 1, y = "1")]
        field_one: String,
    }

    impl Test {
        fn new(field_one: String) -> Self {
            Test { field_one }
        }
    }

    #[derive(cynic::Enum, Clone, Copy)]
    #[cynic(graphql_type = "Dessert", rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum Dessert {
        Cheesecake,
        IceCream,
    }

    #[derive(cynic::InlineFragments)]
    #[cynic(graphql_type = "MyUnionType")]
    pub enum MyUnionType {
        Test(Test),
        Nested(Nested),
    }
}

impl cynic::QueryRoot for query_dsl::TestStruct {}

mod test {

    type JSON = serde_json::Value;

    // A custom scalars.
    pub struct DateTime {}

    impl cynic::Scalar for DateTime {
        fn decode(_: &serde_json::Value) -> Result<Self, json_decode::DecodeError> {
            Ok(DateTime {})
        }
        fn encode(&self) -> Result<serde_json::Value, ::cynic::SerializeError> {
            todo!()
        }
    }

    // Another custom scalar
    struct Upload;

    //cynic::query_dsl!("cms-schema.gql");
}
