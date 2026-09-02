unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 32, align = 8)]
        pub type Context: Copy;

        #[symbol = "_ui_impl_not_copy_new"]
        pub fn new() -> Self::Context;
    }

    macro foo_impl(path = $crate);
}

struct MyImpl;

impl Foo for MyImpl {
    type Context = String;

    fn new() -> String {
        String::new()
    }
}

foo_impl!(MyImpl);

fn main() {}
