fn main() {
    println!("Hello, world!");
}

mod query_dsl {
    // A custom scalar
    struct Upload;

    cynic::query_dsl!("cynic/examples/simple.graphql");
}

mod test {
    pub struct DateTime {}

    impl cynic::Scalar for DateTime {
        fn decode(value: &serde_json::Value) -> Result<Self, json_decode::DecodeError> {
            Ok(DateTime {})
        }
    }

    cynic::query_dsl!("cms-schema.gql");
}
