//! `#[opaque]` under `cfg_attr`: the first attribute whose predicate holds wins.

unitrait::unitrait! {
    /// A test trait whose opaque layouts depend on `cfg`s.
    #[symbol_prefix = "_unitrait_test_cfg_attr"]
    pub trait Layouts {
        /// Several predicates hold; the first one wins, over the fallback too.
        #[cfg_attr(any(), opaque(size = 5, align = 1))]
        #[cfg_attr(all(), opaque(size = 6, align = 1))]
        #[cfg_attr(all(), opaque(size = 9, align = 1))]
        #[opaque(size = 7, align = 1)]
        pub type First: Copy;

        /// No predicate holds, so the fallback applies.
        #[cfg_attr(any(), opaque(size = 5, align = 1))]
        #[cfg_attr(not(test), opaque(size = 6, align = 1))]
        #[opaque(size = 7, align = 1)]
        pub type Fallback: Copy;

        /// No fallback is needed when a predicate holds. The size is rounded up to the
        /// alignment, and the implementation may use all of it.
        #[cfg_attr(target_os = "none", opaque(size = 1, align = 1))]
        #[cfg_attr(test, opaque(size = 5, align = 4))]
        pub type NoFallback: Copy;

        pub fn first() -> Self::First;
        pub fn fallback() -> Self::Fallback;
        pub fn no_fallback() -> Self::NoFallback;
        pub fn no_fallback_get(v: &Self::NoFallback) -> u64;
    }

    /// Set the global implementation.
    macro layouts_impl(path = $crate);
}

struct MyImpl;

impl Layouts for MyImpl {
    type First = [u8; 6];
    type Fallback = [u8; 7];
    // 8 bytes: fills the storage of `size = 5, align = 4` rounded up.
    type NoFallback = [u32; 2];

    fn first() -> [u8; 6] {
        [1; 6]
    }

    fn fallback() -> [u8; 7] {
        [2; 7]
    }

    fn no_fallback() -> [u32; 2] {
        [0x1234_5678, 0x9abc_def0]
    }

    fn no_fallback_get(v: &[u32; 2]) -> u64 {
        (v[0] as u64) << 32 | v[1] as u64
    }
}

layouts_impl!(MyImpl);

#[test]
fn test_cfg_attr_layouts() {
    assert_eq!(core::mem::size_of::<LayoutsFirst>(), 6);
    assert_eq!(core::mem::align_of::<LayoutsFirst>(), 1);
    assert_eq!(core::mem::size_of::<LayoutsFallback>(), 7);
    assert_eq!(core::mem::size_of::<LayoutsNoFallback>(), 8);
    assert_eq!(core::mem::align_of::<LayoutsNoFallback>(), 4);
}

#[test]
fn test_cfg_attr_dispatch() {
    let _ = first();
    let _ = fallback();
    let v = no_fallback();
    assert_eq!(no_fallback_get(&v), 0x1234_5678_9abc_def0);
}
