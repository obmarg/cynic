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
pub struct MyString(String);
cynic::impl_scalar!(MyString, schema::String);

pub struct MyInt(i64);
cynic::impl_scalar!(MyInt, schema::Int);

pub struct MyFloat(f64);
cynic::impl_scalar!(MyFloat, schema::Float);

pub struct MyBool(bool);
cynic::impl_scalar!(MyBool, schema::Boolean);

pub struct MyId(cynic::Id);
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
