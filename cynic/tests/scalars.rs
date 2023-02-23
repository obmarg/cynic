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
