unitrait::unitrait! {
    pub trait Foo {
        #[symbol = "_ui_self_ref_mut"]
        pub fn foo(&mut self) -> u32;
    }

    macro foo_impl(path = $crate);
}

fn main() {}
