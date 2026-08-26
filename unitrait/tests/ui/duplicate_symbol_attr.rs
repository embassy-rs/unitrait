unitrait::unitrait! {
    pub trait Foo {
        #[symbol = "_ui_dup_symbol_a"]
        #[symbol = "_ui_dup_symbol_b"]
        pub fn foo() -> u32;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
