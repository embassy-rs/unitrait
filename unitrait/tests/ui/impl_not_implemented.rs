unitrait::unitrait! {
    pub trait Foo {
        #[symbol = "_ui_impl_not_implemented"]
        pub fn foo() -> u32;
    }

    macro foo_impl(path = $crate);
}

struct MyImpl;

foo_impl!(MyImpl);

fn main() {}
