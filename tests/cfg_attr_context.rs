use std::sync::atomic::{AtomicU32, Ordering};

unitrait::unitrait! {
    /// A cfg-gated accumulator with caller-allocated state.
    pub trait CfgAccumulator {
        /// Opaque storage: 32 bytes when testing, 64 bytes otherwise.
        #[cfg_attr(test, opaque(size = 32, align = 8))]
        #[cfg_attr(not(test), opaque(size = 64, align = 16))]
        #[symbol = "_unitrait_test_cfg_acc_drop"]
        pub type Context;

        /// Returns a fresh accumulator seeded with `seed`.
        #[symbol = "_unitrait_test_cfg_acc_init"]
        pub fn cfg_acc_init(seed: u64) -> Self::Context;

        /// Returns the accumulated value.
        #[symbol = "_unitrait_test_cfg_acc_get"]
        pub fn cfg_acc_get(ctx: &mut Self::Context) -> u64;
    }

    /// Set the global cfg accumulator implementation.
    macro test_cfg_accumulator_impl(path = $crate);
}

static CFG_DROPS: AtomicU32 = AtomicU32::new(0);

struct MyCfgAccumulator;

struct MyCfgState {
    sum: u32,
}

impl Drop for MyCfgState {
    fn drop(&mut self) {
        CFG_DROPS.fetch_add(1, Ordering::Relaxed);
    }
}

impl CfgAccumulator for MyCfgAccumulator {
    type Context = MyCfgState;

    fn cfg_acc_init(seed: u64) -> MyCfgState {
        MyCfgState { sum: seed as u32 }
    }

    fn cfg_acc_get(ctx: &mut MyCfgState) -> u64 {
        ctx.sum as u64
    }
}

test_cfg_accumulator_impl!(MyCfgAccumulator);

#[test]
fn test_cfg_attr_context_size() {
    // When cfg(test) is active, the first variant (size=32, align=8) is selected.
    assert_eq!(core::mem::size_of::<CfgAccumulatorContext>(), 32);
    assert_eq!(core::mem::align_of::<CfgAccumulatorContext>(), 8);
}

#[test]
fn test_cfg_attr_context_dispatch() {
    let mut ctx = cfg_acc_init(100);
    assert_eq!(cfg_acc_get(&mut ctx), 100);
}

#[test]
fn test_cfg_attr_context_drop() {
    let before = CFG_DROPS.load(Ordering::Relaxed);
    let mut ctx = cfg_acc_init(1);
    assert_eq!(CFG_DROPS.load(Ordering::Relaxed), before);
    drop(ctx);
    assert_eq!(CFG_DROPS.load(Ordering::Relaxed), before + 1);
}
