unitrait::unitrait! {
    pub trait Foo {
        #[symbol = "_ui_impl_wrong_signature"]
        pub fn foo(x: u32) -> u32;
    }

    macro foo_impl(path = $crate);
}

struct MyImpl;

impl Foo for MyImpl {
    fn foo(x: u16) -> u32 {
        x as u32
    }
}

foo_impl!(MyImpl);

fn main() {}
