unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 8, align = 4)]
        #[clone_symbol = "_ui_impl_not_clone_clone"]
        pub type Context: Clone;

        #[symbol = "_ui_impl_not_clone_new"]
        pub fn new() -> Self::Context;
    }

    macro foo_impl(path = $crate);
}

pub struct NotClone(pub u32);

struct MyImpl;

impl Foo for MyImpl {
    type Context = NotClone;

    fn new() -> NotClone {
        NotClone(0)
    }
}

foo_impl!(MyImpl);

fn main() {}
