#[test]
fn ui_test_inlinefragments() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/cases/inline-fragment-exhaustiveness.rs");
    t.compile_fail("tests/cases/inline-fragment-fallback-validation.rs");
    t.compile_fail("tests/cases/enum-guess-validation.rs");
}
