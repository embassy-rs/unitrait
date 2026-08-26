unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 8, align = 4)]
        #[symbol = "_ui_ref_return_drop"]
        pub type Context;

        #[symbol = "_ui_ref_return_get"]
        pub fn get() -> &Self::Context;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
