fn main() {}

mod schema {
    cynic::use_schema!("../../../../schemas/simple.graphql");
}

#[derive(cynic::InlineFragments)]
#[cynic(
    schema_path = "../../../../schemas/simple.graphql",
    graphql_type = "MyUnionType",
    exhaustive
)]
enum MyFailingUnionType {
    Nested(Dummy),
    // This is missing `TestStruct`
    #[cynic(fallback)]
    Other,
}

#[derive(cynic::InlineFragments)]
#[cynic(
    schema_path = "../../../../schemas/simple.graphql",
    graphql_type = "MyUnionType",
    exhaustive
)]
enum MyOtherFailingUnionType {
    Nested(Dummy),
    TestStruct(Dummy),
    RandomOtherThing(Dummy),
    #[cynic(fallback)]
    Other,
}

/// The derives above should fail before any useful code is output so we can just
/// use this dummy type.
pub struct Dummy;
