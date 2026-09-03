unitrait::unitrait! {
    pub trait FooDriver {
        #[opaque(size = 8, align = 4)]
        #[drop_symbol = "_ui_opaque_not_copy_drop"]
        pub type Context: Drop;

        #[symbol = "_ui_opaque_not_copy_new"]
        fn new() -> Self::Context;

        #[symbol = "_ui_opaque_not_copy_take"]
        fn take(ctx: Self::Context);
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

struct MyImpl;

impl FooDriver for MyImpl {
    type Context = u32;

    fn new() -> u32 {
        0
    }

    fn take(_ctx: u32) {}
}

foo_impl!(MyImpl);

fn main() {
    let ctx = Foo::new();
    Foo::take(ctx);
    // Without a `Copy` bound the opaque type isn't `Copy`, even though `u32` is.
    Foo::take(ctx);
}
