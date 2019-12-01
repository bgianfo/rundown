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
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::integer_arithmetic)]

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
pub use rundown_ref::RundownError;
