// Copyright 2019 Brian Gianforcaro

use pretty_assertions::assert_eq;
use run_down::{RundownError, RundownGuard, RundownRef};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[cfg(test)]
#[macro_use]
extern crate assert_impl;

//-------------------------------------------------------------------
// Test: test_rundown_guard_implements_drop
//
// Description:
//  Test that RundownGuard implements Drop.
//
#[test]
fn test_rundown_guard_implements_drop() {
    assert_impl!(Drop: RundownGuard);
}

//-------------------------------------------------------------------
// Test: test_acquisition_when_not_rundown
//
// Description:
//  Test that acquisition of run-down protection succeeds
//  when the RundownRef has not yet been marked for run-down.
//
#[test]
fn test_acquisition_when_not_rundown() {
    let rundown = RundownRef::new();

    let result = rundown.try_acquire();
    assert!(result.is_ok());

    let _guard: RundownGuard = result.unwrap();
}

//-------------------------------------------------------------------
// Test: test_acquisition_when_rundown
//
// Description:
//  Test that acquisition of run-down protection fails
//  when the RundownRef has successfully been run-down.
//
#[test]
fn test_acquisition_when_rundown() {
    let rundown_ref = RundownRef::new();

    // Rundown the object.
    rundown_ref.wait_for_rundown();

    let result = rundown_ref.try_acquire();
    assert_eq!(result.err(), Some(RundownError::RundownInProgress));
}

//-------------------------------------------------------------------
// Test: test_re_init
//
// Description:
//  Test that re_init works in the designed mode.
//
#[test]
fn test_re_init() {
    // Setup and completely run-down the object.
    let rundown_ref = RundownRef::new();
    rundown_ref.wait_for_rundown();

    // Rundown on the object should succeed again.
    rundown_ref.re_init();
    rundown_ref.wait_for_rundown();
}

//-------------------------------------------------------------------
// Test: test_re_init_panic_without_rundown
//
// Description:
//  Test that re_init without running-down the object panics.
//
#[test]
#[should_panic]
fn test_re_init_panic_without_rundown() {
    let rundown_ref = RundownRef::new();

    // Re-init should panic as run-down has not occurred.
    rundown_ref.re_init();
}

//-------------------------------------------------------------------
// Test: test_re_init_panic_on_ref
//
// Description:
//  Test that re_init with and outstanding protection panics.
//
#[test]
#[should_panic]
fn test_re_init_panic_on_ref() {
    let rundown_ref = RundownRef::new();
    let _guard = rundown_ref.try_acquire().unwrap();

    // Re-init should panic as run-down has not occurred.
    rundown_ref.re_init();
}

//-------------------------------------------------------------------
// Test: test_usage_with_concurrency
//
// Description:
//  A simple test case to validate the usage of RundownRef
//  and RundownGuard in the wild. We spawn a few threads all attempting
//  to acquire rundown protection, and holding it for a small duration.
//  In parallel we attempt to run-down the ref so that no one can acquire
//  rundown protection, and we wait for that to complete. On my machine
//  most of the threads succeed, while one of the last threads fail as
//  the object was successfully run-down.
//
#[test]
fn test_usage_with_concurrency() {
    let rundown = Arc::new(RundownRef::new());

    for _ in 0..20 {
        let rundown_clone = Arc::clone(&rundown);

        thread::spawn(move || match rundown_clone.try_acquire() {
            Ok(_) => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(_) => return,
        });
    }

    rundown.wait_for_rundown();
}
