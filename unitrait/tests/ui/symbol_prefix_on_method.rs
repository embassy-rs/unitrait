unitrait::unitrait! {
    pub trait Foo {
        #[symbol_prefix = "_ui_prefix_on_method"]
        pub fn foo() -> u32;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
