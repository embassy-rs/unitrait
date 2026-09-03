//! Trait items may appear in any order, like in a real Rust trait: methods may
//! reference opaque associated types declared later.

unitrait::unitrait! {
    /// A counter whose opaque type is declared after the methods that use it.
    pub trait CounterDriver {
        /// Returns a fresh counter.
        #[symbol = "_unitrait_test_order_new"]
        fn counter_new() -> Self::State;

        /// Increments the counter and returns the new value.
        #[symbol = "_unitrait_test_order_bump"]
        fn counter_bump(state: &mut Self::State) -> u32;

        /// Opaque storage for the counter state.
        #[opaque(size = 8, align = 4)]
        #[drop_symbol = "_unitrait_test_order_drop"]
        pub type State: Drop;

        /// Reads the counter without modifying it.
        #[symbol = "_unitrait_test_order_get"]
        fn counter_get(state: &Self::State) -> u32;
    }

    pub struct Counter;

    /// Set the global counter implementation.
    macro test_counter_impl(path = $crate);
}

struct MyCounter;

impl CounterDriver for MyCounter {
    type State = u32;

    fn counter_new() -> u32 {
        0
    }

    fn counter_bump(state: &mut u32) -> u32 {
        *state += 1;
        *state
    }

    fn counter_get(state: &u32) -> u32 {
        *state
    }
}

test_counter_impl!(MyCounter);

#[test]
fn test_interleaved_item_order() {
    let mut state = Counter::counter_new();
    assert_eq!(Counter::counter_get(&state), 0);
    assert_eq!(Counter::counter_bump(&mut state), 1);
    assert_eq!(Counter::counter_bump(&mut state), 2);
    assert_eq!(Counter::counter_get(&state), 2);
}
