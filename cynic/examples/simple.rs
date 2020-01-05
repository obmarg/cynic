fn main() {
    println!("Hello, world!");
}

mod query_dsl {
    cynic::query_dsl!("cynic/examples/simple.graphql");
}
