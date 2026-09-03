use std::sync::atomic::{AtomicU32, Ordering};

unitrait::unitrait! {
    /// A test accumulator with caller-allocated state.
    pub trait Accumulator {
        /// Opaque storage for the accumulator state.
        #[opaque(size = 32, align = 8)]
        #[drop_symbol = "_unitrait_test_acc_drop"]
        pub type Context: Drop;

        /// Returns a fresh accumulator seeded with `seed`.
        #[symbol = "_unitrait_test_acc_init"]
        pub fn acc_init(seed: u64) -> Self::Context;

        /// Adds every byte of `data` to the accumulator.
        #[symbol = "_unitrait_test_acc_update"]
        pub fn acc_update(ctx: &mut Self::Context, data: &[u8]);

        /// Returns the accumulated value.
        #[symbol = "_unitrait_test_acc_get"]
        pub fn acc_get(ctx: &mut Self::Context) -> u64;
    }

    /// Set the global accumulator implementation.
    macro test_accumulator_impl(path = $crate);
}

static DROPS: AtomicU32 = AtomicU32::new(0);

struct MyAccumulator;

// Deliberately smaller and less aligned than the opaque type, with drop glue.
struct MyState {
    sum: u32,
}

impl Drop for MyState {
    fn drop(&mut self) {
        DROPS.fetch_add(1, Ordering::Relaxed);
    }
}

impl Accumulator for MyAccumulator {
    type Context = MyState;

    fn acc_init(seed: u64) -> MyState {
        MyState { sum: seed as u32 }
    }

    fn acc_update(ctx: &mut MyState, data: &[u8]) {
        for &b in data {
            ctx.sum = ctx.sum.wrapping_add(b as u32);
        }
    }

    fn acc_get(ctx: &mut MyState) -> u64 {
        ctx.sum as u64
    }
}

test_accumulator_impl!(MyAccumulator);

#[test]
fn test_context_dispatch() {
    assert_eq!(core::mem::size_of::<AccumulatorContext>(), 32);
    assert_eq!(core::mem::align_of::<AccumulatorContext>(), 8);

    let mut ctx = acc_init(100);
    acc_update(&mut ctx, &[1, 2, 3]);
    assert_eq!(acc_get(&mut ctx), 106);
    acc_update(&mut ctx, &[4]);
    assert_eq!(acc_get(&mut ctx), 110);

    let mut other = acc_init(0);
    assert_eq!(acc_get(&mut other), 0);
    assert_eq!(acc_get(&mut ctx), 110);
}

#[test]
fn test_context_drop() {
    let before = DROPS.load(Ordering::Relaxed);
    let mut ctx = acc_init(1);
    acc_update(&mut ctx, &[1]);
    assert_eq!(DROPS.load(Ordering::Relaxed), before);
    drop(ctx);
    assert_eq!(DROPS.load(Ordering::Relaxed), before + 1);
}
