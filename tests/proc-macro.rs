#[test]
fn compile_tests() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/compile-tests/simple.rs");
    cases.compile_fail("tests/compile-tests/bad_cmp_op.rs");
    cases.compile_fail("tests/compile-tests/bad_cmp_expr.rs");
    cases.compile_fail("tests/compile-tests/bad_op.rs");
    cases.compile_fail("tests/compile-tests/bad_nested.rs");
    cases.compile_fail("tests/compile-tests/eq_range.rs");
    cases.compile_fail("tests/compile-tests/garbage.rs");
    cases.compile_fail("tests/compile-tests/bad_add_var_args.rs");
    cases.pass("tests/compile-tests/add_var.rs");
    cases.compile_fail("tests/compile-tests/user_cuts_deprecated.rs");
}
