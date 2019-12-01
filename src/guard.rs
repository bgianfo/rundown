use crate::rundown_ref::RundownRef;

/// An RAII implementation a "scoped lock" pattern, but specialized
/// to the needs of run-down protection. When this structure is dropped
/// (falls out of scope), the rundown protection reference that was
/// previously acquired will be released.
///
/// This structure is created by the `try_acquire` method on `RundownRef`.
///
/// This type attempts to follow the RAII guidance here: 
/// <https://github.com/rust-unofficial/patterns/blob/master/patterns/RAII.md>
///
pub struct RundownGuard<'r> {
    run_down_ref: &'r RundownRef,
}

impl<'r> RundownGuard<'r> {
    pub const fn new(run_down_ref: &'r RundownRef) -> RundownGuard<'r> {
        Self { run_down_ref }
    }
}

impl<'r> Drop for RundownGuard<'r> {
    fn drop(&mut self) {
        self.run_down_ref.release()
    }
}
