fn main() {
    use queries::{TestArgs, TestStruct};

    //println!("{}", cynic::Query::new<TestStruct>(TestArgs {}));
}

mod query_dsl {
    type Json = serde_json::Value;

    cynic::query_dsl!("cynic/examples/simple.graphql");
}

use cynic::selection_set;

#[cynic::query_module(
    schema_path = "cynic/examples/simple.graphql",
    query_module = "query_dsl"
)]
mod queries {
    use super::query_dsl;

    #[derive(Clone, cynic::FragmentArguments)]
    pub struct TestArgs {}

    #[derive(cynic::QueryFragment)]
    #[cynic(graphql_type = "TestStruct", argument_struct = "TestArgs")]
    pub struct TestStruct {
        #[cynic_arguments(x = Some(1), y = Some("1".to_string()))]
        field_one: String,
        nested: Nested,
        opt_nested: Option<Nested>,
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
        #[cynic_arguments(x = Some(1), y = Some("1".to_string()))]
        field_one: String,
    }

    impl Test {
        fn new(field_one: String) -> Self {
            Test { field_one }
        }
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
    }

    // Another custom scalar
    struct Upload;

    //cynic::query_dsl!("cms-schema.gql");
}
