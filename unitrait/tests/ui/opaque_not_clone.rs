unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 8, align = 4)]
        pub type Context;

        #[symbol = "_ui_opaque_not_clone_new"]
        pub fn new() -> Self::Context;
    }

    macro foo_impl(path = $crate);
}

struct MyImpl;

impl Foo for MyImpl {
    type Context = u32;

    fn new() -> u32 {
        0
    }
}

foo_impl!(MyImpl);

fn main() {
    let ctx = new();
    // Without a `Clone` bound the opaque type isn't `Clone`, even though `u32` is.
    let _ = ctx.clone();
}
