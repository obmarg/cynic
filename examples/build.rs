fn main() {
    cynic_codegen::register_schema("starwars")
        .from_sdl_file("../schemas/starwars.schema.graphql")
        .unwrap()
        .as_default()
        .unwrap();
    cynic_codegen::register_schema("github")
        .from_sdl_file("../schemas/github.graphql")
        .unwrap();
}
