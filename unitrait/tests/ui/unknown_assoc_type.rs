unitrait::unitrait! {
    pub trait Foo {
        #[symbol = "_ui_unknown_assoc_new"]
        pub fn new() -> Self::Context;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
