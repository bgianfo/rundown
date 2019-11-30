use rundown::{RundownGuard, RundownRef};

#[test]
fn basic_usage() {
    let rr: RundownRef = RundownRef::new();

    let result = rr.try_acquire();
    assert!(result.is_ok());

    let _guard: RundownGuard = result.unwrap();
}
