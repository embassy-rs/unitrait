//! Two unitraits in the same module may use the same method and associated type names,
//! since callers reach them through their dispatch types.

unitrait::unitrait! {
    /// A 32-bit checksum.
    #[symbol_prefix = "_unitrait_test_same_names_sum32"]
    pub trait Sum32Driver {
        /// Opaque checksum state.
        #[opaque(size = 8, align = 4)]
        pub type Context: Copy;

        /// Returns a fresh state.
        fn init() -> Self::Context;

        /// Absorbs `data`.
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Returns the checksum.
        fn finalize(ctx: Self::Context) -> u32;
    }

    /// The global 32-bit checksum.
    pub struct Sum32;

    macro test_sum32_impl(path = $crate);
}

unitrait::unitrait! {
    /// A 64-bit checksum.
    #[symbol_prefix = "_unitrait_test_same_names_sum64"]
    pub trait Sum64Driver {
        /// Opaque checksum state.
        #[opaque(size = 16, align = 8)]
        pub type Context: Copy;

        /// Returns a fresh state.
        fn init() -> Self::Context;

        /// Absorbs `data`.
        fn update(ctx: &mut Self::Context, data: &[u8]);

        /// Returns the checksum.
        fn finalize(ctx: Self::Context) -> u64;
    }

    /// The global 64-bit checksum.
    pub struct Sum64;

    macro test_sum64_impl(path = $crate);
}

struct MyImpl;

impl Sum32Driver for MyImpl {
    type Context = u32;

    fn init() -> u32 {
        0
    }

    fn update(ctx: &mut u32, data: &[u8]) {
        for &b in data {
            *ctx = ctx.wrapping_add(b as u32);
        }
    }

    fn finalize(ctx: u32) -> u32 {
        ctx
    }
}

impl Sum64Driver for MyImpl {
    type Context = u64;

    fn init() -> u64 {
        1 << 32
    }

    fn update(ctx: &mut u64, data: &[u8]) {
        for &b in data {
            *ctx = ctx.wrapping_add(b as u64);
        }
    }

    fn finalize(ctx: u64) -> u64 {
        ctx
    }
}

test_sum32_impl!(MyImpl);
test_sum64_impl!(MyImpl);

#[test]
fn test_same_method_names_dispatch_separately() {
    let mut a = Sum32::init();
    Sum32::update(&mut a, &[1, 2, 3]);
    assert_eq!(Sum32::finalize(a), 6);

    let mut b = Sum64::init();
    Sum64::update(&mut b, &[1, 2, 3]);
    assert_eq!(Sum64::finalize(b), (1 << 32) + 6);

    assert_eq!(core::mem::size_of::<Sum32Context>(), 8);
    assert_eq!(core::mem::size_of::<Sum64Context>(), 16);
}

// Both dispatch types implement their trait, so generic code accepts them.
fn checksum<T: Sum32Driver>(data: &[u8]) -> u32 {
    let mut ctx = T::init();
    T::update(&mut ctx, data);
    T::finalize(ctx)
}

#[test]
fn test_dispatch_types_implement_their_traits() {
    assert_eq!(checksum::<Sum32>(b"ab"), 195);
    assert_eq!(checksum::<MyImpl>(b"ab"), 195);
}
