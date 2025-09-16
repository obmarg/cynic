#![allow(non_snake_case)]

mod schema {
    cynic::use_schema!("../schemas/renames.graphql");
}

#[derive(Debug, PartialEq, cynic::QueryFragment)]
#[cynic(
    schema_path = "../schemas/renames.graphql",
    graphql_type = "TestStruct"
)]
struct DefaultCamelCaseNames {
    #[cynic(rename = "is_snake_case")]
    renamed: i32,
    is_camel_case: i32,
}

#[derive(Debug, PartialEq, cynic::QueryFragment)]
#[cynic(
    schema_path = "../schemas/renames.graphql",
    graphql_type = "TestStruct",
    rename_all = "camelCase"
)]
struct ExplicitCamelCaseNames {
    #[cynic(rename = "is_snake_case")]
    renamed: i32,
    is_camel_case: i32,
}

#[derive(Debug, PartialEq, cynic::QueryFragment)]
#[cynic(
    schema_path = "../schemas/renames.graphql",
    graphql_type = "TestStruct",
    rename_all = "snake_case"
)]
struct SnakeCaseNames {
    isSnakeCase: i32,
    #[cynic(rename = "isCamelCase")]
    renamed: i32,
}

#[derive(Debug, PartialEq, cynic::QueryFragment)]
#[cynic(
    schema_path = "../schemas/renames.graphql",
    graphql_type = "TestStruct",
    rename_all = "none"
)]
struct NoRenameAllNames {
    is_snake_case: i32,
    #[cynic(rename = "isCamelCase")]
    renamed: i32,
}

#[test]
fn test_renames() {
    let val = serde_json::json!({
        "is_snake_case": 1,
        "isCamelCase": 2
    });

    assert_eq!(
        serde_json::from_value::<DefaultCamelCaseNames>(val.clone()).unwrap(),
        DefaultCamelCaseNames {
            renamed: 1,
            is_camel_case: 2,
        }
    );

    assert_eq!(
        serde_json::from_value::<ExplicitCamelCaseNames>(val.clone()).unwrap(),
        ExplicitCamelCaseNames {
            renamed: 1,
            is_camel_case: 2,
        }
    );

    assert_eq!(
        serde_json::from_value::<SnakeCaseNames>(val.clone()).unwrap(),
        SnakeCaseNames {
            isSnakeCase: 1,
            renamed: 2,
        }
    );

    assert_eq!(
        serde_json::from_value::<NoRenameAllNames>(val.clone()).unwrap(),
        NoRenameAllNames {
            is_snake_case: 1,
            renamed: 2,
        }
    );
}
