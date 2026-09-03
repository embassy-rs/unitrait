unitrait::unitrait! {
    pub trait FooDriver {
        #[symbol = "_ui_dup_symbol_a"]
        #[symbol = "_ui_dup_symbol_b"]
        fn foo() -> u32;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
