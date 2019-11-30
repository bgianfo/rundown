use crate::guard::RundownGuard;
use lazy_init::Lazy;
use rsevents::{Awaitable, ManualResetEvent, State};
use std::result::Result;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

#[derive(Debug)]
pub enum RundownError {
    /// The rundown reference is already in the middle of being rundown.
    RundownInProgress,
}

bitflags! {
    struct RundownFlags: u64 {
        const RUNDOWN_IN_PROGRESS = 0xF000000000000000;
    }
}

impl RundownFlags {
    #[inline]
    pub fn is_rundown_in_progress(&self) -> bool {
        self.contains(RundownFlags::RUNDOWN_IN_PROGRESS)
    }

    #[inline]
    pub fn set_rundown_in_progress(&self) -> u64 {
        self.bits | RundownFlags::RUNDOWN_IN_PROGRESS.bits
    }

    #[inline]
    pub fn get_ref(&self) -> u64 {
        self.bits ^ RundownFlags::RUNDOWN_IN_PROGRESS.bits
    }

    #[inline]
    pub fn is_ref_zero(&self) -> bool {
        self.get_ref() == 0
    }

    #[inline]
    pub fn add_ref(&self) -> u64 {
        return self.bits + 1;
    }

    #[inline]
    pub fn dec_ref(&self) -> u64 {
        return self.bits - 1;
    }
}

#[inline]
fn to_flags(bits: u64) -> RundownFlags {
    // We need to be able to preserve our refcount bits.
    //
    unsafe { RundownFlags::from_bits_unchecked(bits) }
}

pub struct RundownRef {
    ref_count: AtomicU64,

    // Avoid allocating the event unless necessary
    // during the wait operation.
    //
    event: Lazy<ManualResetEvent>,
}

const ORDERING_VAL: Ordering = Ordering::SeqCst;

impl RundownRef {
    /// Initializes a new ['RundownRef'].
    #[inline]
    pub fn new() -> RundownRef {
        RundownRef {
            ref_count: AtomicU64::new(0),
            event: Lazy::new(),
        }
    }

    /// Attempts to acquire rundown protection on this RundownRef,
    /// returns the ['RundownGuard'] which holds the refcount, or
    /// returns an error if the object is already being rundown.
    pub fn try_acquire(&self) -> Result<RundownGuard<'_>, RundownError> {
        match self.try_acquire_internal() {
            Ok(_) => Ok(RundownGuard::new(self)),
            Err(er) => Err(er),
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

    /// Blocks thread execution until their are no outstanding reference
    /// counts taken on the ['RundownRef'], and the internal representation
    /// has been marked with ['RundownFlags::RUNDOWN_IN_PROGRESS'] to signal
    /// that no other thread can safely acquire a reference count afterwards.
    pub fn wait_for_rundown(&self) {
        let mut current = self.load_flags();

        loop {
            // Create the event as it appears other thread
            // are going to need to release their refs for
            // this thread to complete the rundown.
            //
            if !current.is_ref_zero() {
                self.event
                    .get_or_create(|| ManualResetEvent::new(State::Unset));
            }

            // Turn on the rundown bit to inform all other threads
            // that rundown is in progress.
            //
            let bits_with_rundown = current.set_rundown_in_progress();

            match self.compare_exchange(current.bits(), bits_with_rundown) {
                Ok(_) => {
                    current = to_flags(bits_with_rundown);
                    break;
                }
                Err(new_current) => current = to_flags(new_current),
            }
        }

        if !current.is_ref_zero() {
            let event = self.event.get().expect("Must have been set");
            event.wait();
        }
    }

    fn try_acquire_internal(&self) -> Result<(), RundownError> {
        let mut current = self.load_flags();

        loop {
            if current.is_rundown_in_progress() {
                return Err(RundownError::RundownInProgress);
            }

            let new_bits_with_ref = current.add_ref();

            match self.compare_exchange(current.bits(), new_bits_with_ref) {
                Ok(_) => return Ok(()),
                Err(new_current) => current = to_flags(new_current),
            }
        }
    }

    #[inline]
    fn load_flags(&self) -> RundownFlags {
        to_flags(self.ref_count.load(ORDERING_VAL))
    }

    /// Internal helper method to make all of the compare exchange loops a bit
    /// more readable by omitting constant parameters.
    #[inline]
    fn compare_exchange(&self, current: u64, new: u64) -> Result<u64, u64> {
        self.ref_count
            .compare_exchange(current, new, ORDERING_VAL, ORDERING_VAL)
    }
}
