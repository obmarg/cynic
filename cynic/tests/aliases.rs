use cynic::QueryBuilder;
use serde_json::json;

#[derive(cynic::QueryFragment, Debug, PartialEq)]
#[cynic(
    schema_path = "../schemas/starwars.schema.graphql",
    graphql_type = "Root"
)]
struct FilmQueryWithExplicitAlias {
    #[arguments(id = cynic::Id::new("ZmlsbXM6MQ=="))]
    #[cynic(rename = "film", alias)]
    a_new_hope: Option<Film>,

    #[arguments(id = cynic::Id::new("ZmlsbXM6Mg=="))]
    #[cynic(rename = "film", alias)]
    empire_strikes_back: Option<Film>,
}

#[derive(cynic::QueryFragment, Debug, PartialEq)]
#[cynic(
    schema_path = "../schemas/starwars.schema.graphql",
    query_module = "schema"
)]
struct Film {
    title: Option<String>,
}

mod schema {
    cynic::use_schema!("../schemas/starwars.schema.graphql");
}

#[test]
fn test_explicit_alias_query_output() {
    let operation = FilmQueryWithExplicitAlias::build(());

    insta::assert_display_snapshot!(operation.query, @r###"
    query {
      film(id: "ZmlsbXM6MQ==",) {
        title
      }
      film(id: "ZmlsbXM6Mg==",) {
        title
      }
    }

    "###);
}

#[test]
fn test_explicit_alias_decoding() {
    assert_eq!(
        serde_json::from_value::<FilmQueryWithExplicitAlias>(json!({
            "a_new_hope": {"title": "A New Hope"},
            "empire_strikes_back": {"title": "The Empire Strikes Back"}
        }))
        .unwrap(),
        FilmQueryWithExplicitAlias {
            a_new_hope: Some(Film {
                title: Some("A New Hope".into()),
            }),
            empire_strikes_back: Some(Film {
                title: Some("The Empire Strikes Back".into())
            })
        }
    );
}

#[derive(cynic::QueryFragment, Debug, PartialEq)]
#[cynic(
    schema_path = "../schemas/starwars.schema.graphql",
    graphql_type = "Root"
)]
struct FilmQueryWithImplicitAlias {
    #[arguments(id = cynic::Id::new("ZmlsbXM6MQ=="))]
    #[cynic(rename = "film")]
    a_new_hope: Option<Film>,

    #[arguments(id = cynic::Id::new("ZmlsbXM6Mg=="))]
    #[cynic(rename = "film")]
    empire_strikes_back: Option<Film>,
}

#[test]
fn test_implicit_alias_query_output() {
    let operation = FilmQueryWithImplicitAlias::build(());

    insta::assert_display_snapshot!(operation.query, @r###"
    query {
      film(id: "ZmlsbXM6MQ==",) {
        title
      }
      film(id: "ZmlsbXM6Mg==",) {
        title
      }
    }

    "###);
}

#[test]
fn test_implicit_alias_decoding() {
    assert_eq!(
        serde_json::from_value::<FilmQueryWithImplicitAlias>(json!({
            "a_new_hope": {"title": "A New Hope"},
            "empire_strikes_back": {"title": "The Empire Strikes Back"}
        }))
        .unwrap(),
        FilmQueryWithImplicitAlias {
            a_new_hope: Some(Film {
                title: Some("A New Hope".into()),
            }),
            empire_strikes_back: Some(Film {
                title: Some("The Empire Strikes Back".into())
            })
        }
    );
}
