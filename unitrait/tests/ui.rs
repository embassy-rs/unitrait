#[cfg(not(miri))]
#[test]
fn ui() {
    let t = trybuild::TestCases::new();

    // Errors when defining a unitrait.
    t.compile_fail("tests/ui/attr_on_trait.rs");
    t.compile_fail("tests/ui/attr_on_macro.rs");
    t.compile_fail("tests/ui/attr_on_method.rs");
    t.compile_fail("tests/ui/method_missing_symbol.rs");
    t.compile_fail("tests/ui/duplicate_symbol_attr.rs");
    t.compile_fail("tests/ui/symbol_not_namevalue.rs");
    t.compile_fail("tests/ui/symbol_not_string.rs");
    t.compile_fail("tests/ui/self_value.rs");
    t.compile_fail("tests/ui/self_ref.rs");
    t.compile_fail("tests/ui/self_ref_mut.rs");
    t.compile_fail("tests/ui/type_with_equals.rs");
    t.compile_fail("tests/ui/opaque_missing_symbol.rs");
    t.compile_fail("tests/ui/opaque_missing_attr.rs");
    t.compile_fail("tests/ui/opaque_missing_align.rs");
    t.compile_fail("tests/ui/opaque_unknown_key.rs");
    t.compile_fail("tests/ui/duplicate_opaque_attr.rs");
    t.compile_fail("tests/ui/duplicate_assoc_type.rs");
    t.compile_fail("tests/ui/opaque_on_method.rs");
    t.compile_fail("tests/ui/unknown_assoc_type.rs");
    t.compile_fail("tests/ui/nested_self.rs");
    t.compile_fail("tests/ui/opaque_ref_return.rs");
    t.compile_fail("tests/ui/opaque_ref_lifetime.rs");
    t.compile_fail("tests/ui/bad_path_keyword.rs");
    t.compile_fail("tests/ui/empty_path.rs");
    t.compile_fail("tests/ui/align_not_power_of_two.rs");

    // Errors when supplying an implementation.
    t.compile_fail("tests/ui/impl_not_implemented.rs");
    t.compile_fail("tests/ui/impl_too_big.rs");
    t.compile_fail("tests/ui/impl_overaligned.rs");
    t.compile_fail("tests/ui/impl_wrong_signature.rs");
    t.compile_fail("tests/ui/impl_not_a_type.rs");
}
