fn main() {}

mod query_dsl {
    cynic::query_dsl!("../../../cynic/src/bin/simple.graphql");
}

#[derive(cynic::InlineFragments)]
#[cynic(
    schema_path = "../../../cynic/src/bin/simple.graphql",
    query_module = "query_dsl"
)]
enum MyUnionType {
    TestStruct(Test),
    Nestde(Nested),
}

#[derive(cynic::QueryFragment)]
#[cynic(
    schema_path = "../../../cynic/src/bin/simple.graphql",
    query_module = "query_dsl"
)]
struct Nested {
    pub a_string: String,
    pub opt_string: Option<String>,
}

#[derive(cynic::QueryFragment)]
#[cynic(
    schema_path = "../../../cynic/src/bin/simple.graphql",
    query_module = "query_dsl",
    graphql_type = "TestStruct"
)]
struct Test {
    pub field_one: String,
}
