fn main() {
    lalrpop::Configuration::new()
        .set_in_dir("../src/parser")
        .set_out_dir("../src/parser")
        .process()
        .unwrap();
}
