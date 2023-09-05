
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/fail/*.rs");
    t.pass("tests/ui/pass/*.rs");
}

#[test]
#[cfg_attr(not(nightly), ignore = "these tests are only supported on rust nightly")]
fn ui_nightly() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/nightly/fail-*.rs");
    t.pass("tests/ui/nightly/pass-*.rs");
}
