unitrait::unitrait! {
    pub trait FooDriver {
        #[opaque(size = 8, align = 4)]
        #[drop_symbol = "_ui_impl_overaligned_drop"]
        pub type Context: Drop;

        #[symbol = "_ui_impl_overaligned_new"]
        fn new() -> Self::Context;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

struct MyImpl;

#[repr(align(8))]
struct Overaligned(u32);

impl FooDriver for MyImpl {
    type Context = Overaligned;

    fn new() -> Overaligned {
        Overaligned(0)
    }
}

foo_impl!(MyImpl);

fn main() {}
