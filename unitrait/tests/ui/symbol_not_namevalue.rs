unitrait::unitrait! {
    pub trait FooDriver {
        #[symbol("_ui_symbol_not_namevalue")]
        fn foo() -> u32;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
