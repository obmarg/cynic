#[test]
fn ui_test_inlinefragments() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/cases/argument-missing-fields.rs");
    t.compile_fail("tests/cases/enum-guess-validation.rs");
    #[cfg(target_os = "macos")] // For some reason this is giving different errors on CI :(
    t.compile_fail("tests/cases/feature-flag-on-non-default.rs");
    t.compile_fail("tests/cases/fragment-guess-validation.rs");
    t.compile_fail("tests/cases/inline-fragment-exhaustiveness.rs");
    t.compile_fail("tests/cases/inline-fragment-fallback-validation.rs");
    t.compile_fail("tests/cases/inputobject-guess-validation.rs");
    t.compile_fail("tests/cases/missing-variable.rs");
    t.compile_fail("tests/cases/rename-failures.rs");
    t.compile_fail("tests/cases/unregistered-schema.rs");
    t.compile_fail("tests/cases/wrong-enum-type.rs");
    t.compile_fail("tests/cases/wrong-scalar-type.rs");
    t.compile_fail("tests/cases/wrong-variable-type.rs");
    t.pass("tests/cases/input-fragment-no-graphql-type.rs");
}
