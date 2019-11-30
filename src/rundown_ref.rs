use crate::guard::RundownGuard;
use std::result::Result;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

#[derive(Debug)]
pub enum RundownError {
    /// The rundown reference is already in the middle of being rundown.
    RundownInProgress,
}

pub struct RundownRef {
    ref_count: AtomicU64,
}

impl RundownRef {
    pub fn new() -> RundownRef {
        RundownRef {
            ref_count: AtomicU64::new(0),
        }
    }

    pub fn try_acquire(&self) -> Result<RundownGuard<'_>, RundownError> {
        if self.try_acquire_internal() {
            Ok(RundownGuard::new(self))
        } else {
            Err(RundownError::RundownInProgress)
        }
    }

    pub fn release(&self) {
        let _new = self.ref_count.fetch_sub(1, Ordering::SeqCst);
    }

    fn try_acquire_internal(&self) -> bool {
        panic!("Ahh!")
    }
}
