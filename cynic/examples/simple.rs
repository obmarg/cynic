fn main() {
    println!("{}", cynic::to_query::<TestStruct>());
}

mod query_dsl {
    type JSON = serde_json::Value;

    cynic::query_dsl!("cynic/examples/simple.graphql");
}

use cynic::selection_set;

struct TestStruct {
    field_one: String,
    nested: Nested,
}

impl TestStruct {
    fn new(field_one: String, nested: Nested) -> Self {
        TestStruct { field_one, nested }
    }
}

struct Nested {
    a_string: String,
}

impl Nested {
    fn new(a_string: String) -> Self {
        Self { a_string }
    }
}

#[derive(cynic::QueryFragment)]
#[cynic(schema_path = "abcd", query_module = "", graphql_type = "")]
struct Test {}

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

impl cynic::QueryFragment for TestStruct {
    type SelectionSet = selection_set::SelectionSet<'static, Self, query_dsl::TestStruct>;

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

impl cynic::QueryFragment for Nested {
    type SelectionSet = selection_set::SelectionSet<'static, Self, query_dsl::Nested>;

    fn selection_set() -> Self::SelectionSet {
        selection_set::map(Nested::new, query_dsl::Nested::a_string())
    }
}

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
