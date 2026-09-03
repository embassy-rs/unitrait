unitrait::unitrait! {
    #[symbol_prefix = 42]
    pub trait FooDriver {
        fn foo() -> u32;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
