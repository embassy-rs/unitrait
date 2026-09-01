unitrait::unitrait! {
    pub trait Bad {
        #[cfg_attr(test, opaque(size = 8, align = 8), extra)]
        #[symbol = "_unitrait_test_bad_drop"]
        pub type Context;

        macro bad_impl(path = $crate);
    }
}
