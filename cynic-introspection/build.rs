fn main() {
    cynic_codegen::register_schema("introspection")
        .from_sdl_file("src/schema.graphql")
        .unwrap()
        .as_default()
        .unwrap();
}
