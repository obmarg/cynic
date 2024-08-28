#[test]
fn cli_tests() {
    trycmd::TestCases::new()
        .case("tests/cases/help/*.toml")
        .case("README.md");
}
