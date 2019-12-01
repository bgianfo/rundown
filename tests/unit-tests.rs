use rundown::{RundownGuard, RundownRef};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[test]
fn test_acquisition_not_rundown() {
    let rr: RundownRef = RundownRef::new();

    let result = rr.try_acquire();
    assert!(result.is_ok());

    let _guard: RundownGuard = result.unwrap();
}

#[test]
fn test_acquisition_already_rundown() {
    let rr: RundownRef = RundownRef::new();

    rr.wait_for_rundown();

    let result = rr.try_acquire();
    assert!(result.is_err());
}

#[test]
fn test_usage_with_concurrency() {
    let rr = Arc::new(RundownRef::new());

    for _ in 0..20 {
        let rr_clone = Arc::clone(&rr);

        thread::spawn(move || {
            let result = rr_clone.try_acquire();

            match result {
                Ok(_) => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(_) => return ,
            }
        });
    }

    rr.wait_for_rundown();
}
