// Copyright 2019 Brian Gianforcaro

use bitflags::bitflags;

bitflags! {
    pub struct RundownFlags: u64 {
        const RUNDOWN_IN_PROGRESS = 0xF000_0000_0000_0000;
    }
}

impl RundownFlags {
    /// Returns true if the run-down in progress flag is set.
    #[inline]
    pub const fn is_rundown_in_progress(self) -> bool {
        self.contains(Self::RUNDOWN_IN_PROGRESS)
    }

    /// Returns true if the run-down in progress flag is not set.
    #[inline]
    pub const fn is_pre_rundown(self) -> bool {
        !self.contains(Self::RUNDOWN_IN_PROGRESS)
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

    /// Returns true if the reference-count is non zero.
    #[inline]
    pub const fn is_ref_active(self) -> bool {
        self.get_ref() > 0
    }

    /// Returns a new reference-count with a incremented reference count.
    #[inline]
    pub fn add_ref(self) -> u64 {
        if let Some(new_value) = self.bits.checked_add(1) {
            new_value
        } else {
            panic!("Incrementing the reference-count would have over-flowed!");
        }
    }

    /// Returns a new reference-count with a decremented reference count.
    #[inline]
    pub fn dec_ref(self) -> u64 {
        if let Some(new_value) = self.bits.checked_sub(1) {
            new_value
        } else {
            panic!("Decrementing the reference-count would have under-flowed!");
        }
    }
}

/// Utility function for converting raw bits to `RundownFlags`.
#[inline]
pub const fn to_flags(bits: u64) -> RundownFlags {
    // To preserve the reference-count bits which are encoded with
    // the flags we need to use the unchecked version. This requires
    // the use of unsafe.
    unsafe { RundownFlags::from_bits_unchecked(bits) }
}

#[cfg(test)]
mod test {
    use super::{to_flags, RundownFlags};

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
        assert_eq!(false, flags.is_ref_active());

        // Validate that add ref works.
        flags = to_flags(flags.add_ref());
        assert_eq!(1, flags.get_ref());
        assert_eq!(false, flags.is_ref_zero());
        assert_eq!(true, flags.is_ref_active());

        // Validate that dec ref works.
        flags = to_flags(flags.dec_ref());
        assert_eq!(0, flags.get_ref());
        assert_eq!(true, flags.is_ref_zero());
        assert_eq!(false, flags.is_ref_active());

        // Rundown bit should not be present.
        assert_eq!(false, flags.is_rundown_in_progress());
        assert_eq!(true, flags.is_pre_rundown());
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
        assert_eq!(false, flags.is_pre_rundown());

        // Incrementing the reference count should work, and preserve flags.
        flags = to_flags(flags.add_ref());
        assert_eq!(1, flags.get_ref());
        assert_eq!(true, flags.is_rundown_in_progress());
        assert_eq!(false, flags.is_pre_rundown());
    }

    //-------------------------------------------------------------------
    // Test: test_rundown_flags_overflow_panic
    //
    // Description:
    //  A test case to validate that reference-count panics on overflow.
    //
    #[test]
    #[should_panic]
    fn test_rundown_flags_overflow_panic() {
        let flags = to_flags(0xFFFF_FFFF_FFFF_FFFF);
        flags.add_ref();
    }

    //-------------------------------------------------------------------
    // Test: test_rundown_flags_underflow_panic
    //
    // Description:
    //  A test case to validate that reference-count panics on underflow.
    //
    #[test]
    #[should_panic]
    fn test_rundown_flags_underflow_panic() {
        let flags = RundownFlags::empty();
        flags.dec_ref();
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
        let flags = to_flags(0xF000_0000_0000_0001);
        assert_eq!(1, flags.get_ref());
        assert_eq!(true, flags.is_rundown_in_progress());
    }
}
