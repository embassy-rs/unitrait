unitrait::unitrait! {
    pub trait FooDriver {
        #[opaque(size = 8, align = 4)]
        #[drop_symbol = "_ui_impl_nested_wrong_signature_drop"]
        pub type Context: Drop;

        #[symbol = "_ui_impl_nested_wrong_signature"]
        fn get(v: u32) -> Option<(Self::Context, u32)>;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

struct MyImpl;

impl FooDriver for MyImpl {
    type Context = u32;

    fn get(v: u32) -> Option<u32> {
        Some(v)
    }
}

foo_impl!(MyImpl);

fn main() {}
