use macro_magic::test::const_str_eq_test;

#[test]
fn test_const_str_eq_test_basic() {
    assert!(const_str_eq_test("Hello, world!", "Hello,world!").is_none());
    assert!(const_str_eq_test("Hello,world!", "Hello, world!").is_none());
}

#[test]
fn test_const_str_eq_test_whitespace() {
    assert!(const_str_eq_test("std = std default, priority = ()default, section =\n(super:: SECTION_NAME)value, unsafe = ()default,", 
    "std = std default, priority = () default, section = (super::SECTION_NAME)value, unsafe = ()default,").is_none());
}
