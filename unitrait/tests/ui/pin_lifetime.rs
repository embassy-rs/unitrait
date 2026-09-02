unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 8, align = 4)]
        #[symbol = "_ui_pin_lifetime_drop"]
        pub type Context;

        #[symbol = "_ui_pin_lifetime_poke"]
        pub fn poke(ctx: Pin<&'static mut Self::Context>);
    }

    macro foo_impl(path = $crate);
}

fn main() {}
