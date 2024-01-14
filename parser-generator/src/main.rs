fn main() {
    lalrpop::Configuration::new()
        .set_in_dir("../cynic-parser/src")
        .set_out_dir("../cynic-parser/src")
        .process()
        .unwrap();
}
