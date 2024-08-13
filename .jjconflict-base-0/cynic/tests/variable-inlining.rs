use cynic::QueryVariableLiterals;

#[derive(QueryVariableLiterals)]
struct TestArgs<'a> {
    #[cynic(skip_serializing_if = "Option::is_none")]
    a_str: Option<&'a str>,
    a_bool: Option<bool>,
    an_input: AnInputType,
}

#[derive(cynic::InputObject)]
#[cynic(schema_path = "../schemas/simple.graphql")]
struct AnInputType {
    favourite_dessert: Option<Dessert>,
}

#[derive(cynic::Enum)]
#[cynic(schema_path = "../schemas/simple.graphql")]
enum Dessert {
    Cheesecake,
    IceCream,
}

#[test]
fn test_the_derive() {
    let args = TestArgs {
        a_str: Some("hello"),
        a_bool: Some(false),
        an_input: AnInputType {
            favourite_dessert: Some(Dessert::Cheesecake),
        },
    };

    assert_eq!(args.get("aStr").unwrap().to_string(), "\"hello\"");

    assert_eq!(args.get("aBool").unwrap().to_string(), "false");

    assert_eq!(
        args.get("anInput").unwrap().to_string(),
        "{favouriteDessert: CHEESECAKE}"
    );
}

#[test]
fn test_the_derive_with_skip_serializing_if() {
    let args = TestArgs {
        a_str: None,
        a_bool: None,
        an_input: AnInputType {
            favourite_dessert: None,
        },
    };

    assert_eq!(args.get("aStr"), None)
}

#[test]
fn test_derive_with_renames() {
    #[derive(QueryVariableLiterals)]
    #[cynic(rename_all = "SCREAMING_SNAKE_CASE")]
    struct TestArgs<'a> {
        a_str: Option<&'a str>,
        #[cynic(rename = "renamedThisYeah")]
        a_bool: Option<bool>,
    }
    let args = TestArgs {
        a_str: Some("hello"),
        a_bool: Some(true),
    };

    assert_eq!(args.get("A_STR").unwrap().to_string(), "\"hello\"");
    assert_eq!(args.get("renamedThisYeah").unwrap().to_string(), "true");
}

mod schema {
    cynic::use_schema!("../schemas/simple.graphql");
}
