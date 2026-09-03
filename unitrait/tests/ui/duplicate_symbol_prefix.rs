unitrait::unitrait! {
    #[symbol_prefix = "_ui_dup_prefix_a"]
    #[symbol_prefix = "_ui_dup_prefix_b"]
    pub trait Foo {
        pub fn foo() -> u32;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
