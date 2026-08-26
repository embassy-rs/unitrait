unitrait::unitrait! {
    pub trait Foo {
        #[opaque(size = 8, align = 4)]
        #[symbol = "_ui_opaque_on_method"]
        pub fn foo() -> u32;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
