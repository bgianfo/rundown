use crate::rundown::RundownRef;

/// An RAII implementation a "scoped lock" pattern, but specialized
/// to the needs of run-down protection. When this structure is dropped
/// (falls out of scope), the rundown protection reference that was
/// previously acquired will be released.
///
/// This structure is created by the [`try_acquire'] method on ['RundownRef'].
///
pub struct RundownGuard {
    rundown_ref: &'static mut RundownRef,
}

impl RundownGuard {
    pub fn new(rundown_ref: &'static mut RundownRef) -> RundownGuard {
        RundownGuard { rundown_ref }
    }
}
