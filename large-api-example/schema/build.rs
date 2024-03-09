/// Register github schema for creating definitions
fn main() {
    cynic_codegen::register_schema("github")
        .from_sdl_file("../../schemas/github.graphql")
        .expect("Failed to find GraphQL Schema");
}