unitrait::unitrait! {
    pub trait FooDriver {
        #[clone_symbol = "_ui_method_with_clone_symbol"]
        fn foo() -> u32;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
