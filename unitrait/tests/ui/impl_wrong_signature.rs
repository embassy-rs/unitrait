unitrait::unitrait! {
    pub trait FooDriver {
        #[symbol = "_ui_impl_wrong_signature"]
        fn foo(x: u32) -> u32;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

struct MyImpl;

impl FooDriver for MyImpl {
    fn foo(x: u16) -> u32 {
        x as u32
    }
}

foo_impl!(MyImpl);

fn main() {}
