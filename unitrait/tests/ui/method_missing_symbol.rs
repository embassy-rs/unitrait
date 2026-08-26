unitrait::unitrait! {
    pub trait Foo {
        pub fn foo() -> u32;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
