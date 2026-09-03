unitrait::unitrait! {
    pub trait FooDriver {
        #[symbol_prefix = "_ui_prefix_on_method"]
        fn foo() -> u32;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
