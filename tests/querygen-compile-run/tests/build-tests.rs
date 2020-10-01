//! The build script for this package outputs some generated code to
//! `tests/generated`.  This file ensures that those files can be
//! built successfully.
use std::fs::read_dir;

#[test]
fn build_generated_code() {
    let t = trybuild::TestCases::new();

    for dir in read_dir("tests/generated").unwrap() {
        let dir = dir.unwrap();
        if dir.path().extension().and_then(|s| s.to_str()) == Some("rs") {
            t.pass(dir.path().to_str().unwrap());
        }
    }
}
