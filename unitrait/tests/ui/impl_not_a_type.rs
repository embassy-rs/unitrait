unitrait::unitrait! {
    pub trait FooDriver {
        #[symbol = "_ui_impl_not_a_type"]
        fn foo() -> u32;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

foo_impl!(struct);

fn main() {}
