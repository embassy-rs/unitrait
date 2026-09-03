unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 8, align = 4)]
        #[drop_symbol = "_ui_opaque_not_copy_drop"]
        pub type Context: Drop;

        #[symbol = "_ui_opaque_not_copy_new"]
        pub fn new() -> Self::Context;

        #[symbol = "_ui_opaque_not_copy_take"]
        pub fn take(ctx: Self::Context);
    }

    macro foo_impl(path = $crate);
}

struct MyImpl;

impl Foo for MyImpl {
    type Context = u32;

    fn new() -> u32 {
        0
    }

    fn take(_ctx: u32) {}
}

foo_impl!(MyImpl);

fn main() {
    let ctx = new();
    take(ctx);
    // Without a `Copy` bound the opaque type isn't `Copy`, even though `u32` is.
    take(ctx);
}
