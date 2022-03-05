fn main() {
    #[allow(unused_imports)]
    use queries::{TestArgs, TestStruct};

    //println!("{}", cynic::Operation::query<TestStruct>(TestArgs {}));
}

mod schema {
    cynic::use_schema!("src/bin/simple.graphql");
}

#[cynic::schema_for_derives(file = "src/bin/simple.graphql")]
mod queries {
    use super::schema;

    #[derive(cynic::QueryVariables)]
    pub struct TestArgs {}

    #[derive(cynic::QueryFragment)]
    #[cynic(variables = "TestArgs")]
    pub struct TestStruct {
        #[arguments(x = 1, y = "1")]
        pub field_one: String,
        pub nested: Nested,
        pub opt_nested: Option<Nested>,
        pub dessert: Option<Dessert>,
    }

    #[derive(cynic::QueryFragment)]
    pub struct Nested {
        pub a_string: String,
        pub opt_string: Option<String>,
    }

    #[derive(cynic::QueryFragment)]
    #[cynic(graphql_type = "TestStruct")]
    pub struct Test {
        #[arguments(x = 1, y = "1")]
        pub field_one: String,
    }

    #[derive(cynic::Enum, Clone, Copy)]
    #[cynic(rename_all = "SCREAMING_SNAKE_CASE")]
    pub enum Dessert {
        Cheesecake,
        IceCream,
    }

    #[derive(cynic::InlineFragments)]
    pub enum MyUnionType {
        TestStruct(Test),
        Nested(Nested),
    }
}

impl cynic::schema::QueryRoot for schema::TestStruct {}
