unitrait::unitrait! {
    pub trait Foo {
        #[symbol = "_ui_self_ref"]
        pub fn foo(&self) -> u32;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
