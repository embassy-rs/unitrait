unitrait::unitrait! {
    pub trait FooDriver {
        #[symbol = 42]
        fn foo() -> u32;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
