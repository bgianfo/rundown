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
/*
#![allow(clippy::implicit_return)]
#![allow(clippy::integer_arithmetic)]
#![allow(clippy::print_stdout)]
#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::match_same_arms)]
// TODO: We should really fix these issues.
#![allow(clippy::use_debug)]
#![allow(clippy::missing_docs_in_private_items)]
*/

#[macro_use]
extern crate bitflags;

extern crate lazy_init;
extern crate rsevents;

mod guard;
mod rundown_ref;

// Expose the interface publicly.
//
pub use guard::RundownGuard;
pub use rundown_ref::RundownRef;
