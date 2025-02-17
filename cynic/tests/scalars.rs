use chrono::Timelike;
use cynic::{http::ReqwestExt, QueryBuilder};
use serde::{de::Error, Deserialize, Serialize};
use serde_json::json;

mod schema {
    cynic::use_schema!("tests/test-schema.graphql");
}

// Make sure we can derive DateTime
#[derive(cynic::Scalar)]
pub struct DateTime(pub chrono::DateTime<chrono::Utc>);

// Make sure we can impl_scalar for third party types.
cynic::impl_scalar!(chrono::DateTime<chrono::Utc>, schema::DateTime);

#[derive(cynic::Scalar)]
#[cynic(graphql_type = "DateTime")]
pub struct DateTimeInner<DT>(pub DT);

// Make sure we can use impl scalar on built in types
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MyString(#[allow(dead_code)] String);
cynic::impl_scalar!(MyString, schema::String);

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MyInt(#[allow(dead_code)] i64);
cynic::impl_scalar!(MyInt, schema::Int);

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MyFloat(#[allow(dead_code)] f64);
cynic::impl_scalar!(MyFloat, schema::Float);

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MyBool(#[allow(dead_code)] bool);
cynic::impl_scalar!(MyBool, schema::Boolean);

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MyId(#[allow(dead_code)] cynic::Id);
cynic::impl_scalar!(MyId, schema::ID);

// Also make sure we can derive built in scalars.

#[derive(cynic::Scalar)]
#[cynic(graphql_type = "String")]
pub struct MyString2(String);

#[derive(cynic::Scalar)]
#[cynic(graphql_type = "Int")]
pub struct MyInt2(i64);

#[derive(cynic::Scalar)]
#[cynic(graphql_type = "Float")]
pub struct MyFloat2(f64);

#[derive(cynic::Scalar)]
#[cynic(graphql_type = "Boolean")]
pub struct MyBool2(bool);

#[derive(cynic::Scalar)]
#[cynic(graphql_type = "ID")]
pub struct MyId2(cynic::Id);

#[tokio::test]
async fn test_custom_scalar_serde_on_argument() {
    use chrono::{DateTime, Utc};
    use graphql_mocks::ResolverContext;

    mod schema {
        cynic::use_schema!("tests/test-schema.graphql");
    }

    cynic::impl_scalar_variable!(DateTime<Utc>, schema::Timestamp);
    cynic::impl_foreign_coercions!(DateTime<Utc>, schema::Timestamp);

    impl<'de> cynic::schema::OutputScalar<'de, schema::Timestamp> for DateTime<Utc> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            chrono::DateTime::from_timestamp(i64::deserialize(deserializer)?, 0)
                .ok_or_else(|| D::Error::custom("invalid timestamp"))
        }
    }

    impl cynic::schema::InputScalar<schema::Timestamp> for DateTime<Utc> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.timestamp().serialize(serializer)
        }
    }

    #[derive(cynic::QueryVariables)]
    pub struct Variables {
        time: DateTime<Utc>,
    }

    #[derive(cynic::QueryFragment)]
    #[cynic(variables = "Variables", schema_path = "tests/test-schema.graphql")]
    pub struct Query {
        #[arguments(input: $time)]
        timestamp_echo: DateTime<Utc>,
    }

    let server = graphql_mocks::DynamicSchema::builder(include_str!("test-schema.graphql"))
        .with_resolver("Query", "timestampEcho", |ctx: ResolverContext<'_>| {
            ctx.args
                .get("input")
                .map(|value| json!(value.u64().unwrap()))
        })
        .into_server_builder()
        .await;

    let time = Utc::now().with_nanosecond(0).unwrap();

    let query = Query::build(Variables { time });

    let response = reqwest::Client::new()
        .post(server.url())
        .run_graphql(query)
        .await
        .unwrap();

    if response.errors.is_some() {
        assert_eq!(response.errors.unwrap().len(), 0);
    }

    assert_eq!(response.data.unwrap().timestamp_echo, time);

    // TODO: Also need a test of a scalar on an InputObject...
}
