fn main() {
    cynic_codegen::register_schema("starwars")
        .from_sdl_file("../schemas/starwars.schema.graphql")
        .unwrap()
        .as_default()
        .unwrap();
}
