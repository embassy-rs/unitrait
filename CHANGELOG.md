# Changelog for unitrait

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## Unreleased - ReleaseDate

- Added marker trait bounds on opaque associated types: `type Name: Send + Sync;`. The
  supported bounds are `Send`, `Sync`, `Unpin`, `UnwindSafe`, `RefUnwindSafe` and `Copy`.
- **Breaking, soundness fix**: opaque types no longer implement `Send`, `Sync`, `Unpin`,
  `UnwindSafe` or `RefUnwindSafe` unless declared as a bound on the associated type.
  Previously they implemented all of them unconditionally, regardless of what the
  implementation's associated type implemented, which let safe code send a `!Send`
  implementation value across threads.
- Added support for `Pin<&Self::Name>` and `Pin<&mut Self::Name>` method parameters.

## 1.1.0 - 2026-08-26

- Added support for associated types.
- Reimplemented as a proc macro.

## 1.0.0 - 2026-08-12

Initial release
