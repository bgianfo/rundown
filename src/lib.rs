// Subscribe to most of the clippy lints.
#![warn(
    clippy::all,
    clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]

// Disable warnings for things we don't care about.
#![allow(unknown_lints)]
#![allow(clippy::implicit_return)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::multiple_inherent_impl)]
#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::module_name_repetitions)]

// TODO: We should really fix these issues.
#![allow(clippy::integer_arithmetic)]

// Import the crates we need to use
#[macro_use]
extern crate bitflags;
extern crate lazy_init;
extern crate rsevents;

///! An implementation of run-down protection in rust.
///!
///! Run-down protection is useful when to re-initialize or destroy a shared resource in a [SMP][smp-link] runtime environment.
///!
///! The pattern has two parts, a means to guarantee the resource is accessible and remains so for
///! the during of it's usage. As well as way to make the resource inaccessible from a point going forward
///! and the ability to wait for all outstanding usages to drain so you can safely perform the required operation. 
///!
///! This crate was inspired by the [run-down protection primitive available in the NT Kernel][nt-run-down-docs]. 
///!
///! [nt-run-down-docs]: https://docs.microsoft.com/en-us/windows-hardware/drivers/kernel/run-down-protection
///! [smp-link]: https://en.wikipedia.org/wiki/Symmetric_multiprocessing
mod guard;
mod rundown_ref;
pub use guard::RundownGuard;
pub use rundown_ref::RundownRef;
pub use rundown_ref::RundownError;
