unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 8, align = 4)]
        #[drop_symbol = "_ui_pin_return_drop"]
        pub type Context: Drop;

        #[symbol = "_ui_pin_return_get"]
        pub fn get() -> Pin<&mut Self::Context>;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
