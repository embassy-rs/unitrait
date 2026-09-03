unitrait::unitrait! {
    pub trait Foo {
        #[drop_symbol = "_ui_method_with_drop_symbol"]
        pub fn foo() -> u32;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
