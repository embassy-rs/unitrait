unitrait::unitrait! {
    pub trait Foo {
        #[clone_symbol = "_ui_method_with_clone_symbol"]
        pub fn foo() -> u32;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
