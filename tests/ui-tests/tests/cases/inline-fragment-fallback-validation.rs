fn main() {}

mod schema {
    cynic::use_schema!("../../../../cynic/src/bin/simple.graphql");
}

#[derive(cynic::InlineFragments)]
#[cynic(
    schema_path = "../../../../cynic/src/bin/simple.graphql",
    graphql_type = "MyUnionType"
)]
enum MyFailingUnionType {
    Nested(Nested),

    // This is allowed, but needs to be a string to work.
    #[cynic(fallback)]
    Other(Nested),
}

#[derive(cynic::QueryFragment)]
#[cynic(schema_path = "../../../../cynic/src/bin/simple.graphql")]
struct Nested {
    pub a_string: String,
    pub opt_string: Option<String>,
}

#[derive(cynic::InlineFragments)]
#[cynic(
    schema_path = "../../../../cynic/src/bin/simple.graphql",
    graphql_type = "MyUnionType"
)]
enum MyOkUnionTYpe {
    Nested(Nested),

    #[cynic(fallback)]
    Other,
}

#[derive(cynic::InlineFragments)]
#[cynic(
    schema_path = "../../../../cynic/src/bin/simple.graphql",
    graphql_type = "MyUnionType"
)]
enum UnionTypeWithStringFallback {
    Nested(Nested),

    #[cynic(fallback)]
    Other(String),
}
