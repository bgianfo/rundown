use crate::rundown_ref::RundownRef;

/// An RAII implementation a "scoped lock" pattern, but specialized
/// to the needs of run-down protection. When this structure is dropped
/// (falls out of scope), the rundown protection reference that was
/// previously acquired will be released.
///
/// This structure is created by the [`try_acquire'] method on ['RundownRef'].
///
pub struct RundownGuard<'a> {
    rundown_ref: &'a RundownRef,
}

impl<'rundown_ref> RundownGuard<'rundown_ref> {
    pub fn new(rundown_ref: &'rundown_ref RundownRef) -> RundownGuard<'rundown_ref> {
        RundownGuard { rundown_ref }
    }
}

impl<'rundown_ref> Drop for RundownGuard<'rundown_ref> {
    fn drop(&mut self) {
        self.rundown_ref.release()
    }
}
