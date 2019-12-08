// Copyright 2019 Brian Gianforcaro

//! Run-down protection as a pattern is useful in situations where re-initialization
//! or destruction of a shared resource is required in a [SMP][smp-link] environment.
//! The pattern has two parts, a means to guarantee the resource is accessible and remains so for
//! the during of it's usage. As well as way to make the resource inaccessible from a point going forward
//! and the ability to wait for all outstanding usages to drain so you can safely perform the required operation.
//!
//! This crate was inspired by the [run-down protection primitive in the NT kernel][nt-run-down-docs].
//! Where it's used in situations such as driver unload, where futher access to the driver
//! needs to be rejected and the unloading thread must wait for inflight acesss to stop before
//! the driver can be completely unload.
//!
//! # Example
//!
//! ```rust
//! use run_down::{
//!     RundownGuard,
//!     RundownRef
//! };
//! use std::sync::Arc;
//! use std::thread;
//! use std::time::Duration;
//!
//! let rundown = Arc::new(RundownRef::new());
//!
//! for i in 1..25 {
//!
//!     let rundown_clone = Arc::clone(&rundown);
//!
//!     thread::spawn(move || {
//!
//!         // Attempt to acquire rundown protection, while the main
//!         // thread could be running down the object as we execute.
//!         //
//!         match rundown_clone.try_acquire() {
//!             Ok(run_down_guard) => {
//!                 println!("{}: Run-down protection acquired.", i);
//!
//!                 // Stall the thread while holding rundown protection.
//!                 thread::sleep(Duration::from_millis(10));
//!             }
//!             Err(m) => {
//!                 println!("{}: Failed to acquire run-down protection - {:?}", i, m);
//!             },
//!         }
//!     });
//! }
//!
//! println!("0: Waiting for rundown to complete");
//! rundown.wait_for_rundown();
//! println!("0: Rundown complete");
//! ```
//!
//! [nt-run-down-docs]: https://docs.microsoft.com/en-us/windows-hardware/drivers/kernel/run-down-protection
//! [smp-link]: https://en.wikipedia.org/wiki/Symmetric_multiprocessing

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
#![allow(clippy::missing_docs_in_private_items)]

mod flags;
mod guard;
mod rundown_ref;

pub use crate::guard::RundownGuard;
pub use crate::rundown_ref::RundownError;
pub use crate::rundown_ref::RundownRef;

// Test examples in the README file.
doc_comment::doctest!("../README.md", readme_examples);
