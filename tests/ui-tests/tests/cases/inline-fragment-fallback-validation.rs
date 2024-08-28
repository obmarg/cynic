fn main() {}

mod schema {
    cynic::use_schema!("../../../../schemas/simple.graphql");
}

#[derive(cynic::InlineFragments)]
#[cynic(
    schema_path = "../../../../schemas/simple.graphql",
    graphql_type = "MyUnionType"
)]
enum MyFailingUnionType {
    Nested(Nested),

    // This is allowed, but needs to be a string to work.
    #[cynic(fallback)]
    Other(Nested),
}

#[derive(cynic::QueryFragment)]
#[cynic(schema_path = "../../../../schemas/simple.graphql")]
struct Nested {
    pub a_string: String,
    pub opt_string: Option<String>,
}

#[derive(cynic::InlineFragments)]
#[cynic(
    schema_path = "../../../../schemas/simple.graphql",
    graphql_type = "MyUnionType"
)]
enum MyOkUnionTYpe {
    Nested(Nested),

    #[cynic(fallback)]
    Other,
}

#[derive(cynic::InlineFragments)]
#[cynic(
    schema_path = "../../../../schemas/simple.graphql",
    graphql_type = "MyUnionType"
)]
enum UnionTypeWithStringFallback {
    Nested(Nested),

    #[cynic(fallback)]
    Other(String),
}
