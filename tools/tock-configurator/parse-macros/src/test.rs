#[test]
fn ui() {
    let tests = trybuild::TestCases::new();
    tests.pass("./tests/01.rs");
    tests.compile_fail("./tests/02.rs");
}
