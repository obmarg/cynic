fn main() {
    lalrpop::Configuration::new()
        .set_in_dir("../src/parser")
        .set_out_dir("../src/parser")
        .process()
        .unwrap();

    lalrpop::Configuration::new()
        .set_in_dir("../src/schema_coordinates")
        .set_out_dir("../src/schema_coordinates")
        .process()
        .unwrap();

    for input in [
        "../src/parser/executable.rs",
        "../src/parser/schema.rs",
        "../src/schema_coordinates/parser.rs",
    ] {
        std::process::Command::new("cargo")
            .args(["fmt", "--", input])
            .spawn()
            .ok();
    }
}
