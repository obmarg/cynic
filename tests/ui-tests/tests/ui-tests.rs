#[test]
fn ui_test_inlinefragments() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/cases/enum-guess-validation.rs");
    t.compile_fail("tests/cases/fragment-guess-validation.rs");
    t.compile_fail("tests/cases/inline-fragment-exhaustiveness.rs");
    t.compile_fail("tests/cases/inline-fragment-fallback-validation.rs");
    t.compile_fail("tests/cases/rename-failures.rs");
    t.compile_fail("tests/cases/inputobject-guess-validation.rs");
    t.compile_fail("tests/cases/wrong-enum-type.rs");
    t.pass("tests/cases/input-fragment-no-graphql-type.rs");
}
