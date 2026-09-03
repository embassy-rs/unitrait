unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 8, align = 4)]
        #[drop_symbol = "_ui_ref_lifetime_drop"]
        pub type Context: Drop;

        #[symbol = "_ui_ref_lifetime_peek"]
        pub fn peek(ctx: &'static Self::Context) -> u32;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
