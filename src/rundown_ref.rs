use crate::guard::RundownGuard;
use lazy_init::Lazy;
use rsevents::{Awaitable, ManualResetEvent, State};
use std::result::Result;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

/// The set of errors returned by methods in the run-down crate.
#[derive(Debug, PartialEq)]
pub enum RundownError {
    /// Rundown is already in progress on this shared object.
    RundownInProgress,
}

bitflags! {
    struct RundownFlags: u64 {
        const RUNDOWN_IN_PROGRESS = 0xF000_0000_0000_0000;
    }
}

impl RundownFlags {

    /// Returns true if the run-down in progress flag is set.
    #[inline]
    pub const fn is_rundown_in_progress(self) -> bool {
        self.contains(Self::RUNDOWN_IN_PROGRESS)
    }

    /// Returns a new reference-count with the run-down
    /// in progress flag set in the upper bits.
    #[inline]
    pub const fn set_rundown_in_progress(self) -> u64 {
        self.bits | Self::RUNDOWN_IN_PROGRESS.bits
    }

    /// Returns just the reference-count encoded in the flags.
    #[inline]
    pub const fn get_ref(self) -> u64 {
        self.bits & (!Self::RUNDOWN_IN_PROGRESS.bits)
    }

    /// Returns true if the reference-count is zero.
    #[inline]
    pub const fn is_ref_zero(self) -> bool {
        self.get_ref() == 0
    }

    /// Returns a new reference-count with incremented reference.
    #[inline]
    pub const fn add_ref(self) -> u64 {
        self.bits + 1
    }

    /// Returns a new reference-count with one decremented reference.
    #[inline]
    pub const fn dec_ref(self) -> u64 {
        self.bits - 1
    }
}

//-------------------------------------------------------------------
// Test: test_rundown_flags_refcount
//
// Description:
//  A test case to validate that the reference counting
//  facilities work correctly, namely add-ref and dec-ref.
//
#[test]
fn test_rundown_flags_refcount() {
    // Initialize an empty bit flags.
    let mut flags = RundownFlags::empty();
    assert_eq!(0, flags.get_ref());
    assert_eq!(true, flags.is_ref_zero());

    // Validate that add ref works.
    flags = to_flags(flags.add_ref());
    assert_eq!(1, flags.get_ref());
    assert_eq!(false, flags.is_ref_zero());

    // Validate that dec ref works.
    flags = to_flags(flags.dec_ref());
    assert_eq!(0, flags.get_ref());
    assert_eq!(true, flags.is_ref_zero());
 
    // Rundown bit should not be present.
    assert_eq!(false, flags.is_rundown_in_progress());
}

//-------------------------------------------------------------------
// Test: test_rundown_flags_set_in_progress
//
// Description:
//  A test case to validate that the bit manipulations responsible
//  for managing reference-count as well as the rundown-bit are
//  correctly implemented and the masking works as required..
//
#[test]
fn test_rundown_flags_set_in_progress() {
    // Initialize an empty bit flags.
    let mut flags = RundownFlags::empty();
    assert_eq!(0, flags.get_ref());

    // Turn on rundown in progress flags
    flags = to_flags(flags.set_rundown_in_progress());

    // Reference count should still be zero.
    assert_eq!(0, flags.get_ref());
    assert_eq!(true, flags.is_rundown_in_progress());

    // Incrementing the reference count should work, and preserve flags.
    flags = to_flags(flags.add_ref());
    assert_eq!(1, flags.get_ref());
    assert_eq!(true, flags.is_rundown_in_progress());
}

/// Utility function for converting raw bits to `RundownFlags`.
#[inline]
const fn to_flags(bits: u64) -> RundownFlags {
    // To preserve the reference-count bits which are encoded with
    // the flags we need to use the unchecked version. This requires
    // the use of unsafe.
    unsafe { RundownFlags::from_bits_unchecked(bits) }
}

//-------------------------------------------------------------------
// Test: test_to_flags
//
// Description:
//  A test case to validate that to_flags correctly round-trips
//  all of the bits, including both the flags and reference count.
//
#[test]
fn test_to_flags() {
    let flags = to_flags(0xF000000000000001);
    assert_eq!(1, flags.get_ref());
    assert_eq!(true, flags.is_rundown_in_progress());
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
    /// unless there is an active reference count when rundown started.
    event: Lazy<ManualResetEvent>,
}

/// Common atomic ordering option for all of our compare exchange, loads and stores.
const ORDERING_VAL: Ordering = Ordering::SeqCst;

impl RundownRef {

    /// Initializes a new [`RundownRef`].
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Re-initialize this instance so it can be used again.
    /// It is only valid to call re_init once the object is
    /// completely run-down, via the wait_for_rundown method.
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
        if !current.is_rundown_in_progress() ||
           !current.is_ref_zero() {
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
        self.ref_count.store(0, ORDERING_VAL);
    }

    /// Attempts to acquire rundown protection on this 'RundownRef',
    /// returns the ['RundownGuard'] which holds the reference count,
    /// or returns an error if the object is already being rundown.
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

    /// Utility function to attempt to try to acquire run-down protection.
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

    /// Utility function to load the current flags atomically so that
    /// we can start our Atomic Compare and Swap loops.
    #[inline]
    fn load_flags(&self) -> RundownFlags {
        to_flags(self.ref_count.load(ORDERING_VAL))
    }

    /// Utility method to make all of the compare exchange loops a bit
    /// more readable by omitting constant parameters.
    #[inline]
    fn compare_exchange(&self, current: u64, new: u64) -> Result<u64, u64> {
        self.ref_count
            .compare_exchange(current, new, ORDERING_VAL, ORDERING_VAL)
    }
}
