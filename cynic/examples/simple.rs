fn main() {
    println!("Hello, world!");
}

mod query_dsl {
    type JSON = serde_json::Value;

    cynic::query_dsl!("cynic/examples/simple.graphql");
}

mod test {

    type JSON = serde_json::Value;

    // A custom scalars.
    pub struct DateTime {}

    impl cynic::Scalar for DateTime {
        fn decode(_: &serde_json::Value) -> Result<Self, json_decode::DecodeError> {
            Ok(DateTime {})
        }
    }

    // Another custom scalar
    struct Upload;

    cynic::query_dsl!("cms-schema.gql");
}
