//! The implementation macro resolves the implementor's type in the caller's scope, so it may
//! share its name with an item of the defining module, including the dispatch type. If the
//! macro's glob import won, the shims would register the dispatch type as the implementation
//! and every call would recurse into itself.

unitrait::unitrait! {
    /// A test trait.
    pub trait CounterDriver {
        /// An opaque type, so the opaque shims resolve the implementor's type too.
        #[opaque(size = 4, align = 4)]
        pub type State: Copy;

        #[symbol = "_unitrait_test_named_like_dispatch_create"]
        fn create(v: u32) -> Self::State;

        #[symbol = "_unitrait_test_named_like_dispatch_get"]
        fn get(s: Self::State) -> u32;
    }

    /// The dispatch type, named `Counter`.
    pub struct Counter;

    macro test_counter_impl(path = $crate);
}

mod imp {
    /// The implementor, also named `Counter`.
    pub struct Counter;

    impl crate::CounterDriver for Counter {
        type State = u32;

        fn create(v: u32) -> u32 {
            v + 1
        }

        fn get(s: u32) -> u32 {
            s
        }
    }

    test_counter_impl!(Counter);
}

#[test]
fn test_implementor_named_like_dispatch_type() {
    assert_eq!(Counter::get(Counter::create(41)), 42);
}
