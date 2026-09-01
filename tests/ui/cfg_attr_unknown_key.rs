unitrait::unitrait! {
    pub trait Bad {
        #[cfg_attr(test, opaque(size = 16, align = 8, bad = 1))]
        #[symbol = "_unitrait_test_bad_drop"]
        pub type Context;

        macro bad_impl(path = $crate);
    }
}
