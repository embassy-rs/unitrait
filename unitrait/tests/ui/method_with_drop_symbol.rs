unitrait::unitrait! {
    pub trait FooDriver {
        #[drop_symbol = "_ui_method_with_drop_symbol"]
        fn foo() -> u32;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
