// Keep snapshots on one feature set: rustc's dyn-compatibility suggestions and
// generated methods vary with the `either` and `alloc` features.
#![cfg(all(feature = "either", feature = "alloc"))]

#[test]
fn macro_ui_contract() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/*.rs");
    tests.pass("tests/ui-pass/*.rs");
}
