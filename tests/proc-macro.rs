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
}