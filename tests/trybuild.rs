#[test]
#[cfg_attr(miri, ignore = "ui tests don't run under miri")]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/fail/*.rs");
    t.pass("tests/ui/pass/*.rs");
}

#[test]
#[cfg_attr(
    not(nightly),
    ignore = "these tests are only supported on rust nightly"
)]
#[cfg_attr(miri, ignore = "ui tests don't run under miri")]
fn ui_nightly() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/nightly/fail-*.rs");
    t.pass("tests/ui/nightly/pass-*.rs");
}
