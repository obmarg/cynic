fn main() {
    use cynic::QueryBuilder;

    println!("{}", TestStruct::build(&TestArgs {}).query);
}

mod schema {
    cynic::use_schema!("src/bin/simple.graphql");
}

#[derive(cynic::FragmentArguments)]
struct TestArgs {}

#[derive(cynic::QueryFragment)]
#[cynic(
    schema_path = "src/bin/simple.graphql",
    query_module = "schema",
    argument_struct = "TestArgs"
)]
struct TestStruct {
    #[arguments(x = Some(1), y = "1")]
    pub field_one: String,
    pub nested: Nested,
    pub opt_nested: Option<Nested>,

    #[arguments(input = AnInputType { favourite_dessert: None })]
    pub field_with_input: Dessert,
}

#[derive(cynic::QueryFragment)]
#[cynic(schema_path = "src/bin/simple.graphql", query_module = "schema")]
struct Nested {
    pub a_string: String,
    pub opt_string: Option<String>,
}

#[derive(cynic::QueryFragment)]
#[cynic(
    schema_path = "src/bin/simple.graphql",
    query_module = "schema",
    graphql_type = "TestStruct"
)]
struct Test {
    #[arguments(x = Some(1), y = Some("1".to_string()))]
    pub field_one: String,
    #[arguments(input = AnInputType { favourite_dessert: None })]
    pub field_with_input: Dessert,
}

#[derive(cynic::InputObject, Clone)]
#[cynic(
    schema_path = "src/bin/simple.graphql",
    query_module = "schema",
    rename_all = "camelCase"
)]
struct AnInputType {
    favourite_dessert: Option<Dessert>,
}

#[derive(cynic::Enum, Clone)]
#[cynic(
    schema_path = "src/bin/simple.graphql",
    query_module = "schema",
    rename_all = "SCREAMING_SNAKE_CASE"
)]
enum Dessert {
    Cheesecake,
    IceCream,
}

#[derive(cynic::InlineFragments)]
#[cynic(schema_path = "src/bin/simple.graphql", query_module = "schema")]
enum MyUnionType {
    TestStruct(Test),
    Nested(Nested),
}

/*
fn query() {
    let query = schema::Query::test_struct(selection_set::map2(
        TestStruct::new,
        schema::TestStruct::field_one(),
        schema::TestStruct::nested(selection_set::map(
            Nested::new,
            schema::Nested::a_string(),
        )),
    ));
}*/

impl cynic::QueryRoot for schema::TestStruct {}

// TODO: Some sort of ToQuery trait
// That's only implemented when QueryFragment::SelectionSet::TypeLock == RootQuery
// TODO: I should figure out how arguments could work?

/*

impl cynic::QueryFragment<'static> for TestStruct {
    type SelectionSet = selection_set::SelectionSet<'static, Self, schema::TestStruct>;
    type Arguments = ArgStruct;

    fn query() -> Self::SelectionSet {
        // TODO: Got to say I'm not that enamoured with this syntax.
        // Is there a better way to write this?
        selection_set::map2(
            TestStruct::new,
            schema::TestStruct::field_one(),
            schema::TestStruct::nested(Nested::selection_set()),
        )
    }
}

impl cynic::QueryFragment<'static> for Nested {
    type SelectionSet = selection_set::SelectionSet<'static, Self, schema::Nested>;

    fn query() -> Self::SelectionSet {
        selection_set::map(Nested::new, schema::Nested::a_string())
    }
}
*/
