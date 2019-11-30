use crate::guard::RundownGuard;
use std::result::Result;
use std::sync::atomic::AtomicI64;

pub enum RundownError {
    /// The rundown reference is already in the middle of being rundown.
    RundownInProgress,
}

pub struct RundownRef {
    ref_count: AtomicI64,
}

impl RundownRef {
    pub fn new() -> RundownRef {
        RundownRef {
            ref_count: AtomicI64::new(0),
        }
    }

    pub fn try_acquire(&'static mut self) -> Result<RundownGuard, RundownError> {
        if self.try_acquire_internal() {
            Ok(RundownGuard::new(self))
        } else {
            Err(RundownError::RundownInProgress)
        }
    }

    fn try_acquire_internal(&self) -> bool {
        panic!("Ahh!")
    }

}

