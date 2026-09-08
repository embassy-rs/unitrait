unitrait::unitrait! {
    pub trait FooDriver {
        #[opaque(size = 8, align = 4)]
        #[drop_symbol = "_ui_nested_pin_return_drop"]
        pub type Context: Drop;

        #[symbol = "_ui_nested_pin_return"]
        fn get() -> (u32, Pin<&mut Self::Context>);
    }

    pub struct Foo;

    macro foo_impl(path = $crate);
}

fn main() {}
