fn main() {
    // Run registered benchmarks.
    divan::main();
}

const SCHEMA: &str = include_str!("../../schemas/github.graphql");

#[divan::bench]
fn schema_registration() -> cynic_codegen::registration::SchemaRegistration<'static> {
    cynic_codegen::register_schema("github")
        .dry_run()
        .from_sdl(SCHEMA)
        .unwrap()
}
