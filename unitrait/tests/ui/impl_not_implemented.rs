unitrait::unitrait! {
    pub trait FooDriver {
        #[symbol = "_ui_impl_not_implemented"]
        fn foo() -> u32;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

struct MyImpl;

foo_impl!(MyImpl);

fn main() {}
