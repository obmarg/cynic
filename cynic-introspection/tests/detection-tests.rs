use cynic::GraphQlResponse;
use cynic_introspection::{CapabilitiesQuery, SpecificationVersion};

#[test]
fn shapshot_test_detection_query() {
    use cynic::QueryBuilder;

    insta::assert_snapshot!(CapabilitiesQuery::build(()).query, @r###"
    query CapabilitiesQuery {
      __type(name: "__Type") {
        fields(includeDeprecated: true) {
          name
        }
      }
    }

    "###);
}

#[test]
fn detection_query_with_2018_response() {
    let capabilities = serde_json::from_str::<GraphQlResponse<CapabilitiesQuery>>(include_str!(
        "detection-responses/2018.json"
    ))
    .unwrap();

    assert_eq!(
        capabilities.data.unwrap().version_supported(),
        SpecificationVersion::June2018
    );
}

#[test]
fn detection_query_with_2021_response() {
    let response = serde_json::from_str::<GraphQlResponse<CapabilitiesQuery>>(include_str!(
        "detection-responses/2021.json"
    ))
    .unwrap();

    assert_eq!(
        response.data.unwrap().version_supported(),
        SpecificationVersion::October2021
    );
}
