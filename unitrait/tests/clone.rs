//! An opaque type with a `Clone` bound is cloned by the implementation, through its own
//! extern symbol.

use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

unitrait::unitrait! {
    /// A test trait handing out cloneable opaque buffers.
    pub trait BuffersDriver {
        /// A growable buffer. Cloning one clones the implementation's value.
        #[opaque(size = 32, align = 8)]
        #[drop_symbol = "_unitrait_test_buf_drop"]
        #[clone_symbol = "_unitrait_test_buf_clone"]
        pub type Buf: Clone + Drop;

        /// A cloneable handle with no drop glue.
        #[opaque(size = 8, align = 4)]
        #[clone_symbol = "_unitrait_test_tag_clone"]
        pub type Tag: Clone;

        #[symbol = "_unitrait_test_buf_new"]
        fn buf_new(first: u32) -> Self::Buf;

        #[symbol = "_unitrait_test_buf_push"]
        fn buf_push(b: &mut Self::Buf, v: u32);

        #[symbol = "_unitrait_test_buf_sum"]
        fn buf_sum(b: &Self::Buf) -> u32;

        #[symbol = "_unitrait_test_buf_len"]
        fn buf_len(b: &Self::Buf) -> usize;

        #[symbol = "_unitrait_test_tag_new"]
        fn tag_new(v: u32) -> Self::Tag;

        #[symbol = "_unitrait_test_tag_get"]
        fn tag_get(t: &Self::Tag) -> u32;
    }

    pub struct Buffers;

    /// Set the global buffer implementation.
    macro test_buffers_impl(path = $crate);
}

static CLONES: AtomicU32 = AtomicU32::new(0);
static DROPS: AtomicU32 = AtomicU32::new(0);

/// Tests run in parallel, so those that count clones and drops take this to keep each
/// other's activity out of their deltas.
static COUNTERS: Mutex<()> = Mutex::new(());

struct MyImpl;

/// Owns a heap allocation, so a byte-wise copy would be wrong: only a real `clone` gives
/// two independently usable values.
struct MyBuf {
    data: Vec<u32>,
}

impl Clone for MyBuf {
    fn clone(&self) -> Self {
        CLONES.fetch_add(1, Ordering::Relaxed);
        MyBuf {
            data: self.data.clone(),
        }
    }
}

impl Drop for MyBuf {
    fn drop(&mut self) {
        DROPS.fetch_add(1, Ordering::Relaxed);
    }
}

impl BuffersDriver for MyImpl {
    type Buf = MyBuf;
    type Tag = u32;

    fn buf_new(first: u32) -> MyBuf {
        MyBuf { data: vec![first] }
    }

    fn buf_push(b: &mut MyBuf, v: u32) {
        b.data.push(v);
    }

    fn buf_sum(b: &MyBuf) -> u32 {
        b.data.iter().sum()
    }

    fn buf_len(b: &MyBuf) -> usize {
        b.data.len()
    }

    fn tag_new(v: u32) -> u32 {
        v
    }

    fn tag_get(t: &u32) -> u32 {
        *t
    }
}

test_buffers_impl!(MyImpl);

fn assert_clone<T: ::core::clone::Clone>() {}

#[test]
fn test_clone_bound_is_implemented() {
    assert_clone::<BuffersBuf>();
    assert_clone::<BuffersTag>();
}

#[test]
fn test_clone_goes_through_the_implementation() {
    let _guard = COUNTERS.lock().unwrap_or_else(|e| e.into_inner());
    let clones = CLONES.load(Ordering::Relaxed);

    let mut a = Buffers::buf_new(1);
    Buffers::buf_push(&mut a, 2);

    let b = a.clone();
    assert_eq!(CLONES.load(Ordering::Relaxed), clones + 1);

    // A deep clone, not a copy of the opaque bytes: the two are independent.
    Buffers::buf_push(&mut a, 3);
    assert_eq!(Buffers::buf_len(&a), 3);
    assert_eq!(Buffers::buf_sum(&a), 6);
    assert_eq!(Buffers::buf_len(&b), 2);
    assert_eq!(Buffers::buf_sum(&b), 3);
}

#[test]
fn test_clone_from_goes_through_clone() {
    let _guard = COUNTERS.lock().unwrap_or_else(|e| e.into_inner());
    let (clones, drops) = (
        CLONES.load(Ordering::Relaxed),
        DROPS.load(Ordering::Relaxed),
    );

    let a = Buffers::buf_new(7);
    let mut b = Buffers::buf_new(0);

    // The default `clone_from` is `*self = source.clone()`, so it clones through the
    // symbol and drops the overwritten value through the drop symbol.
    b.clone_from(&a);
    assert_eq!(CLONES.load(Ordering::Relaxed), clones + 1);
    assert_eq!(DROPS.load(Ordering::Relaxed), drops + 1);
    assert_eq!(Buffers::buf_sum(&b), 7);
}

#[test]
fn test_clones_are_dropped() {
    let _guard = COUNTERS.lock().unwrap_or_else(|e| e.into_inner());
    let drops = DROPS.load(Ordering::Relaxed);
    {
        let a = Buffers::buf_new(1);
        let _b = a.clone();
    }
    assert_eq!(DROPS.load(Ordering::Relaxed), drops + 2);
}

#[test]
fn test_clone_without_drop() {
    let t = Buffers::tag_new(5);
    let u = t.clone();
    assert_eq!(Buffers::tag_get(&t), 5);
    assert_eq!(Buffers::tag_get(&u), 5);
    assert!(!core::mem::needs_drop::<BuffersTag>());
}
