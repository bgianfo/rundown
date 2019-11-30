use rundown::{RundownGuard, RundownRef};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[test]
fn basic_usage() {
    let rr: RundownRef = RundownRef::new();

    let result = rr.try_acquire();
    assert!(result.is_ok());

    let _guard: RundownGuard = result.unwrap();
}

#[test]
fn parallel_usage() {
    let rr = Arc::new(RundownRef::new());

    for _ in 0..20 {
        let rr_clone = Arc::clone(&rr);

        thread::spawn(move || {
            let result = rr_clone.try_acquire();

            match result {
                Ok(_) => {
                    println!("Got guard!");
                    thread::sleep(Duration::from_millis(10));
                }
                Err(_) => println!("Rundown in progress!"),
            }
        });
    }

    println!("Waiting for rundown...");
    rr.wait_for_rundown();
    println!("Rundown complete.");
}
