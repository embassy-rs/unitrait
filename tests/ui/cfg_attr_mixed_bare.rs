unitrait::unitrait! {
    pub trait Bad {
        #[opaque(size = 16, align = 8)]
        #[cfg_attr(test, opaque(size = 32, align = 8))]
        #[symbol = "_unitrait_test_bad_drop"]
        pub type Context;

        macro bad_impl(path = $crate);
    }
}
