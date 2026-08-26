unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 8, align = 4)]
        #[symbol = "_ui_impl_too_big_drop"]
        pub type Context;

        #[symbol = "_ui_impl_too_big_new"]
        pub fn new() -> Self::Context;
    }

    macro foo_impl(path = $crate);
}

struct MyImpl;

impl Foo for MyImpl {
    type Context = [u8; 9];

    fn new() -> [u8; 9] {
        [0; 9]
    }
}

foo_impl!(MyImpl);

fn main() {}
