# unitrait

Traits with a single global implementation, resolved at link time.

A "unitrait" is a trait that has exactly one implementation across the whole
crate tree. It can be called from anywhere in the tree without carrying
generic parameters or instances around, without `dyn`, and produces link-time
errors if there is no implementation or more than one.

This is achieved by dispatching calls through `extern "Rust"` functions: the
defining crate declares (and calls) functions by symbol name, and the
implementing crate exports them. The linker matches them up.

The use case is allowing pluggalbe "drivers" for for foundational, process-wide facilities where generics would be too viral and `dyn` too costly. For example, it's used for the `embassy-time` driver and
the `embassy-executor` pender and trace hooks.

## Example

Define a unitrait in the crate that *calls* the functionality:

```rust
unitrait::unitrait! {
    /// A driver for the frobnicator.
    pub trait Driver {
        /// Returns the current frobnication level.
        #[symbol = "_frob_level"]
        pub fn level() -> u32;
    }

    /// Set the global frobnicator driver.
    macro frob_driver_impl(path = $crate);
}
```

This expands to:

- The trait, exactly as written.
- One free function per method (here `pub fn level() -> u32`) that calls the
  global implementation through the extern symbol.
- A macro (here `frob_driver_impl!`) that implementor crates use to register
  a type as the global implementation.

Implement it in exactly one crate in the tree:

```rust,ignore
struct MyDriver;

impl frob::Driver for MyDriver {
    fn level() -> u32 { 42 }
}

frob::frob_driver_impl!(MyDriver);
```

Now `frob::level()` works from any crate in the tree.

See the documentation of the `unitrait!` macro for the full details.
