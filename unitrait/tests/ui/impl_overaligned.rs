unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 8, align = 4)]
        #[symbol = "_ui_impl_overaligned_drop"]
        pub type Context;

        #[symbol = "_ui_impl_overaligned_new"]
        pub fn new() -> Self::Context;
    }

    macro foo_impl(path = $crate);
}

struct MyImpl;

#[repr(align(8))]
struct Overaligned(u32);

impl Foo for MyImpl {
    type Context = Overaligned;

    fn new() -> Overaligned {
        Overaligned(0)
    }
}

foo_impl!(MyImpl);

fn main() {}
