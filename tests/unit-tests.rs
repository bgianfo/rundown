use rundown::{RundownGuard, RundownRef, RundownError};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use pretty_assertions::{assert_eq, assert_ne};

#[macro_use]
extern crate assert_impl;

//-------------------------------------------------------------------
// Test: test_rundown_guard_implements_drop
//
// Description:
//  Test that ['RundownGuard'] implements Drop.
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
// Test: test_acquisition_when_already_rundown
//
// Description:
//  Test that acquisition of run-down protection fails
//  when the RundownRef has successfully been run-down.
//
#[test]
fn test_acquisition_when_already_rundown() {
    let rundown_ref = RundownRef::new();

    // Rundown the object.
    rundown_ref.wait_for_rundown();

    let result = rundown_ref.try_acquire();
    assert_eq!(result.err(), Some(RundownError::RundownInProgress));
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

        thread::spawn(move || {
            match rundown_clone.try_acquire() {
                Ok(_) => { thread::sleep(Duration::from_millis(10)); }
                Err(_) => return,
            }
        });
    }

    rundown.wait_for_rundown();
}
