// Copyright 2019 Brian Gianforcaro

use crate::{flags::to_flags, flags::RundownFlags, guard::RundownGuard};
use lazy_init::Lazy;
use rsevents::{Awaitable, ManualResetEvent, State};
use std::{result::Result, sync::atomic::AtomicU64, sync::atomic::Ordering};

/// The set of errors returned by methods in the run-down crate.
#[derive(Debug, PartialEq)]
pub enum RundownError {
    /// Rundown is already in progress on this shared object.
    RundownInProgress,
}

/// Tracks the status of run-down protection for an object.
/// The type would be embedded in the object needing run-down protection.
#[derive(Default)]
pub struct RundownRef {
    /// The reference count used to track the threads that currently have
    /// outstanding run-down protection request being tracked by this object.
    ///
    /// The reference count holds two parts, the actual count in the lower bits
    /// and the flags bit in the most significant bit of the u64. The flags and
    /// reference count interpretation logic is encapsulated in the RundownFlags
    /// type. It has the logic to correctly mask and fetch the required bits.
    ///
    /// We need to bit-pack the flags with the reference count, as we need a single
    /// atomic type that we can use to implement the interlocked operations which
    /// provide the thread safety guaranteed by this type.
    ref_count: AtomicU64,

    /// The event used to signal the thread waiting for rundown that
    /// rundown is now complete.
    ///
    /// The event is lazy initialized to avoid allocating the event
    /// unless there is an active reference count when rundown starts.
    event: Lazy<ManualResetEvent>,
}

impl RundownRef {
    /// Initializes a new [`RundownRef`].
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Re-initialize this instance so it can be used again.
    /// It is only valid to call `re_init` once the object is
    /// completely run-down, via the `wait_for_rundown` method.
    ///
    /// # Important
    ///
    /// The moment this method returns, new rundown protection
    /// requests can succeed. You must perform all re-initialization
    /// of the shared object the run-down protection is guarding
    /// before you call this method.
    pub fn re_init(&self) {
        let current = self.load_flags();

        // Validate that the object in the correct state.
        //
        // TODO: Ideally we should have another bit to represent
        // rundown being complete vs run-down in progress. It would
        // give us a more clear state transition.
        //
        if current.is_pre_rundown() || current.is_ref_active() {
            panic!("Attempt to re-init before rundown is complete");
        }

        // Reset the event if it was previously lazily created so it
        // can be used again in the future. If the event doesn't exist
        // yet, then there is nothing to do.
        if let Some(event) = self.event.get() {
            event.reset();
        }

        // Zero the reference count to make the object ready for use.
        //
        // Note: Once this store completes then new instances of run-down
        // protection will be able to be acquired immediately. All
        // validation and re-initialization needs to occur before this point.
        self.ref_count.store(0, Ordering::Release);
    }

    /// Attempts to acquire rundown protection on this [`RundownRef`],
    /// returns the [`RundownGuard`] which holds the reference count,
    /// or returns an error if the object is already being rundown.
    ///
    /// # Errors
    ///
    /// Will return `Err` if the rundown is already in progress on the object.
    ///
    pub fn try_acquire(&self) -> Result<RundownGuard<'_>, RundownError> {
        let mut current = self.load_flags();

        loop {
            if current.is_rundown_in_progress() {
                return Err(RundownError::RundownInProgress);
            }

            let new_bits_with_ref = current.add_ref();

            match self.compare_exchange(current.bits(), new_bits_with_ref) {
                Ok(_) => return Ok(RundownGuard::new(self)),
                Err(new_current) => current = to_flags(new_current),
            }
        }
    }

    /// Release previously acquired rundown protection.
    pub fn release(&self) {
        let mut current = self.load_flags();

        loop {
            let bits_with_decrement = current.dec_ref();

            match self.compare_exchange(current.bits(), bits_with_decrement) {
                Ok(_) => {
                    current = to_flags(bits_with_decrement);
                    break;
                }
                Err(new_current) => current = to_flags(new_current),
            }
        }

        if current.is_ref_zero() && current.is_rundown_in_progress() {
            let event = self.event.get().expect("Must have been set");
            event.set();
        }
    }

    /// Blocks thread execution until there are no outstanding reference
    /// counts taken on the [`RundownRef`], and the internal representation
    /// has been marked with [`RundownFlags::RUNDOWN_IN_PROGRESS`] to signal
    /// that no other thread can safely acquire a reference count afterwards.
    ///
    /// # Important
    ///
    /// - This method is not thread safe, it must only be called by one thread.
    ///
    /// - This method is however idempotent, it can be called multiple times.
    ///
    pub fn wait_for_rundown(&self) {
        let mut current = self.load_flags();

        loop {
            // If there are outstanding protection reference-counts
            // then create the event. At this point it appears that
            // other threads need to release their protection for
            // this thread to complete the rundown.
            if current.is_ref_active() {
                self.event
                    .get_or_create(|| ManualResetEvent::new(State::Unset));
            }

            // Turn on the rundown bit to inform all other threads
            // that rundown is currently in progress.
            let bits_with_rundown = current.set_rundown_in_progress();

            match self.compare_exchange(current.bits(), bits_with_rundown) {
                Ok(_) => {
                    current = to_flags(bits_with_rundown);
                    break;
                }
                Err(new_current) => current = to_flags(new_current),
            }
        }

        if current.is_ref_active() {
            let event = self.event.get().expect("Must have been set");
            event.wait();
        }
    }

    /// Load the current flags atomically, for use in the start of all
    /// atomic compare and exchange loops in this implementation..
    #[inline]
    fn load_flags(&self) -> RundownFlags {
        // We use Relaxed ordering, as the value is always
        // going to be checked by the compare_exchange later
        // in the loop.
        to_flags(self.ref_count.load(Ordering::Relaxed))
    }

    /// Readability wrapper around atomic compare exchange.
    #[inline]
    fn compare_exchange(&self, current: u64, new: u64) -> Result<u64, u64> {
        self.ref_count
            .compare_exchange(current, new, Ordering::Acquire, Ordering::Relaxed)
    }
}

#[cfg(test)]
mod test {
    use super::RundownRef;
    use std::sync::Arc;
    use std::thread;

    //-------------------------------------------------------------------
    // Test: test_wait_when_protected
    //
    // Description:
    //  Test that `wait_for_rundown` correctly run-down protection fails
    //
    // Notes:
    //  This test needs access to the reference count directly to work.
    //
    #[test]
    #[allow(clippy::unwrap_used)]
    fn wait_when_protected() {
        let rundown = Arc::new(RundownRef::new());

        // Acquire protection.
        let guard = rundown.try_acquire().unwrap();

        // Launch a thread to wait for rundown.
        let rundown_clone = Arc::clone(&rundown);
        let waiter = thread::spawn(move || {
            rundown_clone.wait_for_rundown();
        });

        // Spin until the rundown bit is set, one set we know
        // that the waiter is going to wait and the signal that
        // the drop below will send.
        while rundown.load_flags().is_pre_rundown() {
            thread::yield_now();
        }

        // Release protection, the waiter should be signaled.
        std::mem::drop(guard);

        waiter.join().unwrap();

        // Verify re-init works after the event is used.
        // TODO: Split out into an independent test.
        rundown.re_init();
    }
}
