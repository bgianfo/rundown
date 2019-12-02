use crate::rundown_ref::RundownRef;

/// An RAII implementation of a "scoped lock" pattern, but specialized
/// to the needs of run-down protection. When this structure is dropped
/// (falls out of scope), the rundown protection reference that was
/// previously acquired is released.
///
/// This structure is created by the `try_acquire` method on `RundownRef`.
///
/// This type attempts to follow the RAII guidance here: 
/// <https://github.com/rust-unofficial/patterns/blob/master/patterns/RAII.md>
pub struct RundownGuard<'r> {

    /// The run-dwon reference that this guard objec points too.
    owned_run_down_ref: &'r RundownRef,
}

impl<'r> RundownGuard<'r> {

    /// Creates a new [`RundownGuard`] which owns an instance of run-down
    /// protection on the [`RundownRef`] provided.
    ///
    /// # Arguments
    ///
    /// * `owned_run_down_ref` - The run-down reference to release when the
    ///                          guard goes out of scope.
    ///
    pub const fn new(owned_run_down_ref: &'r RundownRef) -> RundownGuard<'r> {
        Self { owned_run_down_ref }
    }
}

impl<'r> Drop for RundownGuard<'r> {
    /// Releases the previously acquired instance of run-down protection.
    fn drop(&mut self) {
        self.owned_run_down_ref.release()
    }
}
