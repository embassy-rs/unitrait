unitrait::unitrait! {
    pub trait Foo {
        #[symbol("_ui_symbol_not_namevalue")]
        pub fn foo() -> u32;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
