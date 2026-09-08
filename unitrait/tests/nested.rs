//! Opaque associated types nested by value in `Option`, `Result`, tuples and arrays, in any
//! combination, in both parameters and return values.

#![allow(clippy::type_complexity)]

use core::pin::{Pin, pin};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

/// An array length named from the defining crate, which the implementation macro must
/// resolve too.
pub const HANDLES: usize = 4;

unitrait::unitrait! {
    /// A test trait passing opaque values around inside composites.
    pub trait NestedDriver {
        /// Opaque state with drop glue, smaller and less aligned than declared.
        #[opaque(size = 32, align = 8)]
        #[drop_symbol = "_unitrait_test_nested_ctx_drop"]
        pub type Context: Drop;

        /// A copyable handle.
        #[opaque(size = 8, align = 4)]
        pub type Handle: Copy;

        /// A cloneable value, cloned by the implementation.
        #[opaque(size = 32, align = 8)]
        #[drop_symbol = "_unitrait_test_nested_shared_drop"]
        #[clone_symbol = "_unitrait_test_nested_shared_clone"]
        pub type Shared: Clone + Drop;

        #[symbol = "_unitrait_test_nested_ctx_new"]
        fn ctx_new(v: u32) -> Self::Context;

        #[symbol = "_unitrait_test_nested_ctx_get"]
        fn ctx_get(ctx: &Self::Context) -> u32;

        #[symbol = "_unitrait_test_nested_handle_new"]
        fn handle_new(v: u32) -> Self::Handle;

        // `Option`.

        #[symbol = "_unitrait_test_nested_opt_new"]
        fn opt_new(v: Option<u32>) -> Option<Self::Context>;

        #[symbol = "_unitrait_test_nested_opt_get"]
        fn opt_get(ctx: Option<&Self::Context>) -> Option<u32>;

        /// Adds one; returns whether there was a value.
        #[symbol = "_unitrait_test_nested_opt_bump"]
        fn opt_bump(ctx: Option<&mut Self::Context>) -> bool;

        /// Consumes the value, dropping it on the implementation side.
        #[symbol = "_unitrait_test_nested_opt_take"]
        fn opt_take(ctx: Option<Self::Context>) -> Option<u32>;

        /// Records the address the state was pinned at the first time, checks it after.
        #[symbol = "_unitrait_test_nested_opt_pin"]
        fn opt_pin(ctx: Option<Pin<&mut Self::Context>>) -> Option<u32>;

        // `Result`.

        #[symbol = "_unitrait_test_nested_res_new"]
        fn res_new(v: u32, ok: bool) -> Result<Self::Context, u32>;

        #[symbol = "_unitrait_test_nested_res_err_new"]
        fn res_err_new(v: u32, ok: bool) -> Result<u32, Self::Context>;

        #[symbol = "_unitrait_test_nested_res_both"]
        fn res_both(r: Result<Self::Context, Self::Handle>) -> u32;

        #[symbol = "_unitrait_test_nested_res_refs"]
        fn res_refs(r: Result<&Self::Context, &mut Self::Context>) -> u32;

        // Tuples.

        #[symbol = "_unitrait_test_nested_pair_new"]
        fn pair_new(a: u32, b: u32) -> (Self::Context, Self::Handle);

        #[symbol = "_unitrait_test_nested_pair_get"]
        fn pair_get(p: (&Self::Context, Self::Handle)) -> (u32, u32);

        /// Passes the context through, in and out, next to a plain value.
        #[symbol = "_unitrait_test_nested_pair_swap"]
        fn pair_swap(p: (Self::Context, u32)) -> (u32, Self::Context);

        #[symbol = "_unitrait_test_nested_single"]
        fn single(p: (Self::Context,)) -> (Self::Context,);

        /// Adds the value to both contexts.
        #[symbol = "_unitrait_test_nested_triple_mut"]
        fn triple_mut(p: (&mut Self::Context, &mut Self::Context, u32));

        // Arrays.

        #[symbol = "_unitrait_test_nested_arr_new"]
        fn arr_new(base: u32) -> [Self::Context; 3];

        #[symbol = "_unitrait_test_nested_arr_sum"]
        fn arr_sum(a: [Self::Context; 3]) -> u32;

        #[symbol = "_unitrait_test_nested_arr_refs"]
        fn arr_refs(a: [&Self::Context; 2]) -> u32;

        #[symbol = "_unitrait_test_nested_arr_handles"]
        fn arr_handles(h: [Self::Handle; HANDLES]) -> u32;

        #[symbol = "_unitrait_test_nested_arr_empty"]
        fn arr_empty(a: [Self::Context; 0]) -> usize;

        // Nesting.

        /// `None`, `Some(Ok)` or `Some(Err)` depending on `v % 3`.
        #[symbol = "_unitrait_test_nested_deep_new"]
        fn deep_new(v: u32) -> Option<Result<(Self::Context, u32), Self::Handle>>;

        #[symbol = "_unitrait_test_nested_deep_get"]
        fn deep_get(v: Option<Result<(Self::Context, u32), Self::Handle>>) -> u32;

        #[symbol = "_unitrait_test_nested_opts_arr_new"]
        fn opts_arr_new(a: u32, b: Option<u32>) -> [Option<Self::Context>; 2];

        #[symbol = "_unitrait_test_nested_opts_arr_sum"]
        fn opts_arr_sum(a: [Option<Self::Context>; 2]) -> u32;

        /// Adds the value to the context, if both are there; returns the new value.
        #[symbol = "_unitrait_test_nested_nested_refs"]
        fn nested_refs(v: Option<(u32, Option<&mut Self::Context>)>) -> u32;

        /// Bumps the second, if there; returns the sum.
        #[symbol = "_unitrait_test_nested_tuple_pins"]
        fn tuple_pins(p: (Pin<&Self::Context>, Option<Pin<&mut Self::Context>>)) -> u32;

        /// Parenthesized types are seen through.
        #[symbol = "_unitrait_test_nested_parens"]
        fn parens(p: ((Self::Context), Option<(Self::Handle)>)) -> (u32, u32);

        // `Clone`.

        #[symbol = "_unitrait_test_nested_shared_new"]
        fn shared_new(v: u32) -> Option<Self::Shared>;

        #[symbol = "_unitrait_test_nested_shared_get"]
        fn shared_get(s: Option<&Self::Shared>) -> u32;
    }

    pub struct Nested;

    /// Set the global implementation.
    macro test_nested_impl(path = $crate);
}

static CREATED: AtomicU32 = AtomicU32::new(0);
static DROPS: AtomicU32 = AtomicU32::new(0);
static SHARED_CLONES: AtomicU32 = AtomicU32::new(0);
static SHARED_DROPS: AtomicU32 = AtomicU32::new(0);

/// Tests run in parallel, so those that count creations and drops take this to keep each
/// other's activity out of their deltas.
static COUNTERS: Mutex<()> = Mutex::new(());

struct MyImpl;

/// Deliberately smaller and less aligned than the opaque type. `check` is the complement
/// of `value`, so a corrupted round trip through the opaque bytes is caught.
struct MyState {
    value: u32,
    check: u32,
    /// Set by `opt_pin` and `tuple_pins` to the address the state was pinned at.
    anchor: *const MyState,
}

impl MyState {
    fn new(value: u32) -> MyState {
        CREATED.fetch_add(1, Ordering::Relaxed);
        MyState {
            value,
            check: !value,
            anchor: core::ptr::null(),
        }
    }

    fn get(&self) -> u32 {
        assert_eq!(self.check, !self.value, "state corrupted");
        self.value
    }

    fn set(&mut self, value: u32) {
        self.value = value;
        self.check = !value;
    }

    fn pinned(self: Pin<&mut Self>) -> &mut Self {
        // SAFETY: nothing is moved out of the state.
        let this = unsafe { self.get_unchecked_mut() };
        if this.anchor.is_null() {
            this.anchor = this;
        } else {
            assert_eq!(
                this.anchor, this as *const MyState,
                "the state moved while pinned"
            );
        }
        this
    }
}

impl Drop for MyState {
    fn drop(&mut self) {
        self.get();
        DROPS.fetch_add(1, Ordering::Relaxed);
    }
}

#[derive(Clone, Copy)]
struct MyHandle {
    lo: u16,
    hi: u16,
}

impl MyHandle {
    fn new(v: u32) -> MyHandle {
        MyHandle {
            lo: v as u16,
            hi: (v >> 16) as u16,
        }
    }

    fn get(self) -> u32 {
        self.lo as u32 | (self.hi as u32) << 16
    }
}

/// Owns a heap allocation, so a byte-wise copy would be wrong.
struct MyShared(Vec<u32>);

impl Clone for MyShared {
    fn clone(&self) -> Self {
        SHARED_CLONES.fetch_add(1, Ordering::Relaxed);
        MyShared(self.0.clone())
    }
}

impl Drop for MyShared {
    fn drop(&mut self) {
        SHARED_DROPS.fetch_add(1, Ordering::Relaxed);
    }
}

impl NestedDriver for MyImpl {
    type Context = MyState;
    type Handle = MyHandle;
    type Shared = MyShared;

    fn ctx_new(v: u32) -> MyState {
        MyState::new(v)
    }

    fn ctx_get(ctx: &MyState) -> u32 {
        ctx.get()
    }

    fn handle_new(v: u32) -> MyHandle {
        MyHandle::new(v)
    }

    fn opt_new(v: Option<u32>) -> Option<MyState> {
        v.map(MyState::new)
    }

    fn opt_get(ctx: Option<&MyState>) -> Option<u32> {
        ctx.map(MyState::get)
    }

    fn opt_bump(ctx: Option<&mut MyState>) -> bool {
        match ctx {
            Some(ctx) => {
                ctx.set(ctx.get() + 1);
                true
            }
            None => false,
        }
    }

    fn opt_take(ctx: Option<MyState>) -> Option<u32> {
        ctx.map(|ctx| ctx.get())
    }

    fn opt_pin(ctx: Option<Pin<&mut MyState>>) -> Option<u32> {
        ctx.map(|ctx| ctx.pinned().get())
    }

    fn res_new(v: u32, ok: bool) -> Result<MyState, u32> {
        if ok { Ok(MyState::new(v)) } else { Err(v) }
    }

    fn res_err_new(v: u32, ok: bool) -> Result<u32, MyState> {
        if ok { Ok(v) } else { Err(MyState::new(v)) }
    }

    fn res_both(r: Result<MyState, MyHandle>) -> u32 {
        match r {
            Ok(ctx) => ctx.get(),
            Err(h) => h.get(),
        }
    }

    fn res_refs(r: Result<&MyState, &mut MyState>) -> u32 {
        match r {
            Ok(ctx) => ctx.get(),
            Err(ctx) => {
                ctx.set(ctx.get() + 100);
                ctx.get()
            }
        }
    }

    fn pair_new(a: u32, b: u32) -> (MyState, MyHandle) {
        (MyState::new(a), MyHandle::new(b))
    }

    fn pair_get(p: (&MyState, MyHandle)) -> (u32, u32) {
        (p.0.get(), p.1.get())
    }

    fn pair_swap(p: (MyState, u32)) -> (u32, MyState) {
        (p.1, p.0)
    }

    fn single(p: (MyState,)) -> (MyState,) {
        p
    }

    fn triple_mut(p: (&mut MyState, &mut MyState, u32)) {
        p.0.set(p.0.get() + p.2);
        p.1.set(p.1.get() + p.2);
    }

    fn arr_new(base: u32) -> [MyState; 3] {
        [
            MyState::new(base),
            MyState::new(base + 1),
            MyState::new(base + 2),
        ]
    }

    fn arr_sum(a: [MyState; 3]) -> u32 {
        a.iter().map(MyState::get).sum()
    }

    fn arr_refs(a: [&MyState; 2]) -> u32 {
        a[0].get() * 1000 + a[1].get()
    }

    fn arr_handles(h: [MyHandle; HANDLES]) -> u32 {
        h.iter().map(|h| h.get()).sum()
    }

    fn arr_empty(a: [MyState; 0]) -> usize {
        a.len()
    }

    fn deep_new(v: u32) -> Option<Result<(MyState, u32), MyHandle>> {
        match v % 3 {
            0 => None,
            1 => Some(Ok((MyState::new(v), v * 2))),
            _ => Some(Err(MyHandle::new(v))),
        }
    }

    fn deep_get(v: Option<Result<(MyState, u32), MyHandle>>) -> u32 {
        match v {
            None => 0,
            Some(Ok((ctx, n))) => ctx.get() + n,
            Some(Err(h)) => h.get(),
        }
    }

    fn opts_arr_new(a: u32, b: Option<u32>) -> [Option<MyState>; 2] {
        [Some(MyState::new(a)), b.map(MyState::new)]
    }

    fn opts_arr_sum(a: [Option<MyState>; 2]) -> u32 {
        a.iter().flatten().map(MyState::get).sum()
    }

    fn nested_refs(v: Option<(u32, Option<&mut MyState>)>) -> u32 {
        match v {
            Some((n, Some(ctx))) => {
                ctx.set(ctx.get() + n);
                ctx.get()
            }
            _ => 0,
        }
    }

    fn tuple_pins(p: (Pin<&MyState>, Option<Pin<&mut MyState>>)) -> u32 {
        let first = p.0.get();
        let second = match p.1 {
            Some(ctx) => {
                let ctx = ctx.pinned();
                ctx.set(ctx.get() + 1);
                ctx.get()
            }
            None => 0,
        };
        first + second
    }

    fn parens(p: (MyState, Option<MyHandle>)) -> (u32, u32) {
        (p.0.get(), p.1.map_or(0, MyHandle::get))
    }

    fn shared_new(v: u32) -> Option<MyShared> {
        Some(MyShared(vec![v; 3]))
    }

    fn shared_get(s: Option<&MyShared>) -> u32 {
        s.map_or(0, |s| s.0.iter().sum())
    }
}

test_nested_impl!(MyImpl);

/// Takes the counters lock and returns a closure yielding the number of `MyState`s created
/// but not yet dropped, relative to when this was called.
fn track_alive() -> (std::sync::MutexGuard<'static, ()>, impl Fn() -> i64) {
    let guard = COUNTERS.lock().unwrap_or_else(|e| e.into_inner());
    let alive = || CREATED.load(Ordering::Relaxed) as i64 - DROPS.load(Ordering::Relaxed) as i64;
    let base = alive();
    (guard, move || alive() - base)
}

#[test]
fn test_option() {
    let (_guard, alive) = track_alive();

    assert!(Nested::opt_new(None).is_none());
    assert_eq!(alive(), 0);

    let mut ctx = Nested::opt_new(Some(5));
    assert_eq!(alive(), 1);
    assert_eq!(Nested::opt_get(ctx.as_ref()), Some(5));
    assert_eq!(Nested::opt_get(None), None);
    assert!(Nested::opt_bump(ctx.as_mut()));
    assert!(!Nested::opt_bump(None));
    assert_eq!(Nested::opt_get(ctx.as_ref()), Some(6));
    assert_eq!(Nested::ctx_get(ctx.as_ref().unwrap()), 6);

    // Consumed and dropped by the implementation, exactly once.
    assert_eq!(Nested::opt_take(ctx), Some(6));
    assert_eq!(alive(), 0);
    assert_eq!(Nested::opt_take(None), None);
    assert_eq!(alive(), 0);

    // Dropped by the caller, exactly once.
    let ctx = Nested::opt_new(Some(7));
    assert_eq!(alive(), 1);
    drop(ctx);
    assert_eq!(alive(), 0);
}

#[test]
fn test_option_pin() {
    let (_guard, alive) = track_alive();
    {
        let mut ctx = pin!(Nested::ctx_new(3));
        assert_eq!(Nested::opt_pin(Some(ctx.as_mut())), Some(3));
        assert_eq!(Nested::opt_pin(Some(ctx.as_mut())), Some(3));
        assert_eq!(Nested::opt_pin(None), None);
        assert_eq!(alive(), 1);
    }
    assert_eq!(alive(), 0);
}

#[test]
fn test_result() {
    let (_guard, alive) = track_alive();

    let ok = Nested::res_new(1, true);
    let err = Nested::res_new(2, false);
    assert_eq!(alive(), 1);
    let Ok(ctx) = ok else { panic!() };
    let Err(2) = err else { panic!() };
    assert_eq!(Nested::res_refs(Ok(&ctx)), 1);
    assert_eq!(Nested::res_both(Ok(ctx)), 1);
    assert_eq!(alive(), 0);

    let ok = Nested::res_err_new(3, true);
    let err = Nested::res_err_new(4, false);
    assert_eq!(alive(), 1);
    let Ok(3) = ok else { panic!() };
    let Err(mut ctx) = err else { panic!() };
    assert_eq!(Nested::res_refs(Err(&mut ctx)), 104);
    assert_eq!(Nested::ctx_get(&ctx), 104);
    // The opaque value moves from one position of a `Result` to another.
    let moved: Result<NestedContext, NestedHandle> = Ok(ctx);
    assert_eq!(Nested::res_both(moved), 104);
    assert_eq!(
        Nested::res_both(Err(Nested::handle_new(0x1234_5678))),
        0x1234_5678
    );
    assert_eq!(alive(), 0);
}

#[test]
fn test_tuples() {
    let (_guard, alive) = track_alive();

    let (ctx, handle) = Nested::pair_new(10, 0xdead_beef);
    assert_eq!(alive(), 1);
    assert_eq!(Nested::pair_get((&ctx, handle)), (10, 0xdead_beef));

    let (n, ctx) = Nested::pair_swap((ctx, 42));
    assert_eq!(n, 42);
    assert_eq!(Nested::ctx_get(&ctx), 10);
    assert_eq!(alive(), 1);

    let (ctx,) = Nested::single((ctx,));
    assert_eq!(Nested::ctx_get(&ctx), 10);
    assert_eq!(alive(), 1);

    let mut a = ctx;
    let mut b = Nested::ctx_new(20);
    Nested::triple_mut((&mut a, &mut b, 5));
    assert_eq!((Nested::ctx_get(&a), Nested::ctx_get(&b)), (15, 25));
    assert_eq!(alive(), 2);

    assert_eq!(Nested::parens((a, Some(Nested::handle_new(9)))), (15, 9));
    assert_eq!(alive(), 1);
    drop(b);
    assert_eq!(alive(), 0);
}

#[test]
fn test_arrays() {
    let (_guard, alive) = track_alive();

    let arr = Nested::arr_new(100);
    assert_eq!(alive(), 3);
    assert_eq!(Nested::arr_refs([&arr[2], &arr[0]]), 102_100);
    assert_eq!(Nested::arr_sum(arr), 303);
    assert_eq!(alive(), 0);

    let handles = [1, 2, 3, 4].map(Nested::handle_new);
    assert_eq!(Nested::arr_handles(handles), 10);
    // `Copy` all the way up.
    assert_eq!(Nested::arr_handles(handles), 10);

    assert_eq!(Nested::arr_empty([]), 0);

    let arr = Nested::arr_new(0);
    assert_eq!(alive(), 3);
    let [a, b, c] = arr;
    drop(b);
    assert_eq!(alive(), 2);
    assert_eq!(Nested::ctx_get(&a) + Nested::ctx_get(&c), 2);
    drop((a, c));
    assert_eq!(alive(), 0);
}

#[test]
fn test_nesting() {
    let (_guard, alive) = track_alive();

    assert!(Nested::deep_new(0).is_none());
    let ok = Nested::deep_new(1);
    let err = Nested::deep_new(2);
    assert_eq!(alive(), 1);
    match &ok {
        Some(Ok((ctx, n))) => {
            assert_eq!(Nested::ctx_get(ctx), 1);
            assert_eq!(*n, 2);
        }
        _ => panic!(),
    }
    assert!(matches!(err, Some(Err(_))));
    assert_eq!(Nested::deep_get(None), 0);
    assert_eq!(Nested::deep_get(ok), 3);
    assert_eq!(Nested::deep_get(err), 2);
    assert_eq!(alive(), 0);

    let both = Nested::opts_arr_new(1, Some(2));
    let one = Nested::opts_arr_new(3, None);
    assert_eq!(alive(), 3);
    assert!(one[1].is_none());
    assert_eq!(Nested::opts_arr_sum(both), 3);
    assert_eq!(Nested::opts_arr_sum(one), 3);
    assert_eq!(alive(), 0);

    let mut ctx = Nested::ctx_new(10);
    assert_eq!(Nested::nested_refs(Some((5, Some(&mut ctx)))), 15);
    assert_eq!(Nested::nested_refs(Some((5, None))), 0);
    assert_eq!(Nested::nested_refs(None), 0);
    assert_eq!(Nested::ctx_get(&ctx), 15);

    {
        let first = pin!(Nested::ctx_new(1));
        let mut second = pin!(Nested::ctx_new(2));
        assert_eq!(
            Nested::tuple_pins((first.as_ref(), Some(second.as_mut()))),
            4
        );
        assert_eq!(
            Nested::tuple_pins((first.as_ref(), Some(second.as_mut()))),
            5
        );
        assert_eq!(Nested::tuple_pins((first.as_ref(), None)), 1);
        assert_eq!(alive(), 3);
    }
    assert_eq!(alive(), 1);
    drop(ctx);
    assert_eq!(alive(), 0);
}

#[test]
fn test_clone_inside_option() {
    let _guard = COUNTERS.lock().unwrap_or_else(|e| e.into_inner());
    let clones = SHARED_CLONES.load(Ordering::Relaxed);
    let drops = SHARED_DROPS.load(Ordering::Relaxed);

    let a = Nested::shared_new(2);
    // `Option<T: Clone>` is `Clone`, going through the implementation's clone.
    let b = a.clone();
    assert_eq!(SHARED_CLONES.load(Ordering::Relaxed), clones + 1);
    assert_eq!(Nested::shared_get(a.as_ref()), 6);
    assert_eq!(Nested::shared_get(b.as_ref()), 6);
    assert_eq!(Nested::shared_get(None), 0);
    drop(a);
    assert_eq!(SHARED_DROPS.load(Ordering::Relaxed), drops + 1);
    drop(b);
    assert_eq!(SHARED_DROPS.load(Ordering::Relaxed), drops + 2);
}

/// The trait itself, with the rewritten signatures, is usable in generic code, both with
/// the dispatch type and with the implementation.
fn via_trait<D: NestedDriver>() -> u32 {
    let ctx: Option<D::Context> = D::opt_new(Some(1));
    let (a, h): (D::Context, D::Handle) = D::pair_new(2, 3);
    let arr: [D::Context; 3] = D::arr_new(4);
    D::opt_take(ctx).unwrap() + D::res_both(Ok(a)) + D::res_both(Err(h)) + D::arr_sum(arr)
}

#[test]
fn test_generic_code() {
    let (_guard, alive) = track_alive();
    assert_eq!(via_trait::<Nested>(), 1 + 2 + 3 + 15);
    assert_eq!(via_trait::<MyImpl>(), 1 + 2 + 3 + 15);
    assert_eq!(alive(), 0);
}
