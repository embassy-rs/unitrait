unitrait::unitrait! {
    pub trait FooDriver {
        #[opaque(size = 8, align = 4)]
        #[drop_symbol = "_ui_pin_return_drop"]
        pub type Context: Drop;

        #[symbol = "_ui_pin_return_get"]
        fn get() -> Pin<&mut Self::Context>;
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
