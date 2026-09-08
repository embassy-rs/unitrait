unitrait::unitrait! {
    pub trait FooDriver {
        #[opaque(size = 8, align = 4)]
        pub type Context;

        #[symbol = "_ui_impl_opaque_wrong_signature"]
        fn get(ctx: &Self::Context) -> u32;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

struct MyImpl;

impl FooDriver for MyImpl {
    type Context = u32;

    fn get(ctx: &u64) -> u32 {
        *ctx as u32
    }
}

foo_impl!(MyImpl);

fn main() {}
