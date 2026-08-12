#![no_std]
#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

/// Define a unitrait: a trait with a single global implementation, resolved at link time.
///
/// This macro takes a trait definition and the name of an "implementation macro", and emits:
///
/// - The trait, as written.
/// - One free function per trait method, with the same name, signature and documentation,
///   which calls the same method on the global implementation. These functions are callable
///   from anywhere in the crate tree, including crates that can't see the implementation.
///   Each free function's visibility is the visibility written before `fn` on the method
///   (private if omitted, see below).
/// - The implementation macro, which registers a type as the global implementation of the
///   trait. It must be invoked exactly once across the whole crate tree; the program will
///   fail to link if it's invoked zero or multiple times.
///
/// # Syntax
///
/// ```
/// pub struct Item(pub u64);
///
/// unitrait::unitrait! {
///     /// A driver for the frobnicator.
///     pub trait Driver {
///         /// Returns the current frobnication level.
///         #[symbol = "_frob_level"]
///         pub fn level() -> u32;
///
///         /// Returns the item associated with `token`.
///         ///
///         /// # Safety
///         ///
///         /// `token` must have been obtained from [`Item::token`].
///         #[symbol = "_frob_item"]
///         pub unsafe fn item(token: u32) -> &'static mut Item;
///     }
///
///     /// Set the global frobnicator driver.
///     macro frob_driver_impl(path = $crate);
/// }
/// # struct MyDriver;
/// # impl Driver for MyDriver {
/// #     fn level() -> u32 { 42 }
/// #     unsafe fn item(_token: u32) -> &'static mut Item { unimplemented!() }
/// # }
/// # frob_driver_impl!(MyDriver);
/// # fn main() { assert_eq!(level(), 42); }
/// ```
///
/// Trait methods:
///
/// - must not have a `self` parameter. If the implementation needs state, it can store it
///   in `static`s.
/// - must each have a `#[symbol = "..."]` attribute specifying the extern symbol name used
///   for that method. Symbol names must be unique across the program: prefix them
///   with your crate's name and version. Changing a method's symbol name or signature is an
///   ABI-breaking change between the defining crate and implementor crates.
/// - may be `unsafe`. The corresponding free function will be `unsafe` too.
/// - may have a visibility (e.g. `pub fn`, `pub(crate) fn`). It sets the visibility of the
///   generated free function only. Omit it to make the free function private, e.g. when
///   only the defining crate should call the unitrait, while the (more visible) trait
///   stays implementable by other crates.
///
/// The `macro NAME(path = PATH);` line sets the name of the generated implementation macro.
/// `PATH` must be the path of the module containing the `unitrait!` invocation, as seen from
/// other crates, starting with `$crate` — e.g. `$crate` if invoked at the crate root, or
/// `$crate::foo::bar` if invoked in the module `foo::bar`. It's used by the implementation
/// macro to name the trait, so the module must be publicly reachable.
///
/// # Implementing a unitrait
///
/// Implement the trait on some type (typically a unit struct), then invoke the
/// implementation macro on that type:
///
/// ```ignore
/// struct MyDriver;
///
/// impl frob::Driver for MyDriver {
///     fn level() -> u32 { 42 }
///     unsafe fn item(token: u32) -> &'static mut frob::Item { ... }
/// }
///
/// frob::frob_driver_impl!(MyDriver);
/// ```
///
/// When implementing from within the defining crate itself, the implementation macro must be
/// invoked by its bare name (textually scoped, like all `macro_rules!` macros), not through a
/// `crate::` path: `#[macro_use]` on the defining module can be used to extend its textual
/// scope to the rest of the crate.
///
/// # Name resolution in method signatures
///
/// The method signatures are pasted verbatim both into the defining crate (for the trait and
/// free functions) and into implementor crates (by the implementation macro). Therefore, the
/// types they name must resolve in both places. The implementation macro expands inside a
/// scope with `use PATH::*;`, so this works out as long as every type named in the signatures
/// is:
///
/// - a primitive, or
/// - an absolute path (`core::...`, `::somecrate::...`), or
/// - publicly reachable at the root of the module containing the `unitrait!` invocation, and
///   named unqualified (like `Item` in the example above). If the type is defined elsewhere,
///   re-export it with `pub use`.
///
/// Because of that `use PATH::*;` glob, if the type passed to the implementation macro has the
/// same name as a public item of the defining module (e.g. a driver struct named exactly like
/// the trait), the bare name is ambiguous inside the macro expansion. Qualify it to disambiguate:
/// `some_driver_impl!(self::Driver)`.
///
/// # Linkage details
///
/// Calls from the free functions to the implementation are done via `extern "Rust"` functions.
///
/// For each method, the free function calls `extern "Rust" { fn "SYMBOL"(...); }`, and the
/// implementation macro exports `#[export_name = "SYMBOL"] fn(...)` which forwards to the
/// trait implementation. The linker resolves the former to the latter. If no crate in the tree
/// invokes the implementation macro, linking fails with an "undefined symbol" error; if more
/// than one does, it fails with a "duplicate symbol" error.
///
/// Because the contract between the two sides is just the symbol name and signature, crates
/// may define and implement the unitrait through *different versions* of the defining crate
/// (as happens during major-version transitions of the defining crate) and still link
/// correctly, as long as the symbol names and signatures match.
#[macro_export]
macro_rules! unitrait {
    (
        $(#[doc $($tdoc:tt)*])*
        $vis:vis trait $name:ident {
            $($body:tt)*
        }

        $(#[doc $($mdoc:tt)*])*
        macro $mac:ident(path = $($path:tt)*);
    ) => {
        $crate::__unitrait_internal! {
            state {
                mac { $(#[doc $($mdoc)*])* $mac }
                path { $($path)* }
                vis { $vis }
                trait { $(#[doc $($tdoc)*])* $name }
            }
            methods {}
            impls {}
            rest { $($body)* }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __unitrait_internal {
    // Terminal case: all methods munched. Emit the trait and the implementation macro.
    (
        state {
            mac { $(#[doc $($mdoc:tt)*])* $mac:ident }
            path { $($path:tt)* }
            vis { $($vis:tt)* }
            trait { $(#[doc $($tdoc:tt)*])* $name:ident }
        }
        methods { $($methods:tt)* }
        impls { $($impls:tt)* }
        rest {}
    ) => {
        $(#[doc $($tdoc)*])*
        $($vis)* trait $name {
            $($methods)*
        }

        $(#[doc $($mdoc)*])*
        #[macro_export]
        macro_rules! $mac {
            ($t:ty) => {
                const _: () = {
                    #[allow(unused_imports)]
                    use $($path)* ::*;

                    $crate::__unitrait_impl_fns! {
                        trait_path { $($path)* :: $name }
                        ty { $t }
                        methods { $($impls)* }
                    }
                };
            };
        }
    };

    // Munch one safe method.
    (
        state {
            mac { $($mac:tt)* }
            path { $($path:tt)* }
            vis { $($vis:tt)* }
            trait { $($traitdecl:tt)* }
        }
        methods { $($methods:tt)* }
        impls { $($impls:tt)* }
        rest {
            $(#[doc $($doc:tt)*])*
            #[symbol = $sym:literal]
            $fvis:vis fn $fname:ident($($arg:ident: $aty:ty),* $(,)?) $(-> $ret:ty)?;
            $($rest:tt)*
        }
    ) => {
        $(#[doc $($doc)*])*
        #[inline]
        $fvis fn $fname($($arg: $aty),*) $(-> $ret)? {
            unsafe extern "Rust" {
                #[link_name = $sym]
                safe fn extern_fn($($arg: $aty),*) $(-> $ret)?;
            }
            extern_fn($($arg),*)
        }

        $crate::__unitrait_internal! {
            state {
                mac { $($mac)* }
                path { $($path)* }
                vis { $($vis)* }
                trait { $($traitdecl)* }
            }
            methods {
                $($methods)*
                $(#[doc $($doc)*])*
                fn $fname($($arg: $aty),*) $(-> $ret)?;
            }
            impls {
                $($impls)*
                { [$sym] fn $fname($($arg: $aty),*) $(-> $ret)? }
            }
            rest { $($rest)* }
        }
    };

    // Munch one unsafe method.
    (
        state {
            mac { $($mac:tt)* }
            path { $($path:tt)* }
            vis { $($vis:tt)* }
            trait { $($traitdecl:tt)* }
        }
        methods { $($methods:tt)* }
        impls { $($impls:tt)* }
        rest {
            $(#[doc $($doc:tt)*])*
            #[symbol = $sym:literal]
            $fvis:vis unsafe fn $fname:ident($($arg:ident: $aty:ty),* $(,)?) $(-> $ret:ty)?;
            $($rest:tt)*
        }
    ) => {
        $(#[doc $($doc)*])*
        #[inline]
        $fvis unsafe fn $fname($($arg: $aty),*) $(-> $ret)? {
            unsafe extern "Rust" {
                #[link_name = $sym]
                fn extern_fn($($arg: $aty),*) $(-> $ret)?;
            }
            unsafe { extern_fn($($arg),*) }
        }

        $crate::__unitrait_internal! {
            state {
                mac { $($mac)* }
                path { $($path)* }
                vis { $($vis)* }
                trait { $($traitdecl)* }
            }
            methods {
                $($methods)*
                $(#[doc $($doc)*])*
                unsafe fn $fname($($arg: $aty),*) $(-> $ret)?;
            }
            impls {
                $($impls)*
                { [$sym] unsafe fn $fname($($arg: $aty),*) $(-> $ret)? }
            }
            rest { $($rest)* }
        }
    };

    // Anything else is a parse error.
    (
        state { $($state:tt)* }
        methods { $($methods:tt)* }
        impls { $($impls:tt)* }
        rest { $($rest:tt)* }
    ) => {
        compile_error!(concat!(
            "unitrait: could not parse this method. Methods must be of the form ",
            "`#[symbol = \"...\"] [vis] [unsafe] fn name(arg: Ty, ...) [-> Ret];`, ",
            "optionally preceded by doc comments, and must not have a `self` parameter. ",
            "Offending tokens: `",
            stringify!($($rest)*),
            "`",
        ));
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __unitrait_impl_fns {
    (
        trait_path { $($path:tt)* }
        ty { $t:ty }
        methods {}
    ) => {};

    // Emit the exported function for one safe method.
    (
        trait_path { $($path:tt)* }
        ty { $t:ty }
        methods {
            { [$sym:literal] fn $fname:ident($($arg:ident: $aty:ty),*) $(-> $ret:ty)? }
            $($rest:tt)*
        }
    ) => {
        #[unsafe(export_name = $sym)]
        fn $fname($($arg: $aty),*) $(-> $ret)? {
            <$t as $($path)*>::$fname($($arg),*)
        }

        $crate::__unitrait_impl_fns! {
            trait_path { $($path)* }
            ty { $t }
            methods { $($rest)* }
        }
    };

    // Emit the exported function for one unsafe method.
    (
        trait_path { $($path:tt)* }
        ty { $t:ty }
        methods {
            { [$sym:literal] unsafe fn $fname:ident($($arg:ident: $aty:ty),*) $(-> $ret:ty)? }
            $($rest:tt)*
        }
    ) => {
        #[unsafe(export_name = $sym)]
        fn $fname($($arg: $aty),*) $(-> $ret)? {
            // SAFETY: forwarded to the caller through the `unsafe fn` free function
            // matching this method, which carries the trait method's safety contract.
            unsafe { <$t as $($path)*>::$fname($($arg),*) }
        }

        $crate::__unitrait_impl_fns! {
            trait_path { $($path)* }
            ty { $t }
            methods { $($rest)* }
        }
    };
}
