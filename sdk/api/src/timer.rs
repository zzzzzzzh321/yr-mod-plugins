//! Signed frame-anchored countdown timer shared by simulation systems.

use serde::{Deserialize, Serialize};

/// Raw sentinel stored while a countdown is paused.
pub const PAUSED_START_FRAME: i32 = -1;

/// A signed countdown anchored to the global simulation frame.
///
/// While running, `duration` is the original countdown length. Pausing
/// replaces it with the remaining length and switches `start_frame` to the
/// paused sentinel; resuming only captures a new frame anchor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CdTimer {
    start_frame: i32,
    duration: i32,
}

impl Default for CdTimer {
    fn default() -> Self {
        Self {
            start_frame: PAUSED_START_FRAME,
            duration: 0,
        }
    }
}

impl CdTimer {
    /// Preserve the two authoritative signed dwords verbatim.
    #[inline]
    pub const fn from_raw(start_frame: i32, duration: i32) -> Self {
        Self {
            start_frame,
            duration,
        }
    }

    /// Construct and start a countdown at `current_frame`.
    #[inline]
    pub const fn started(current_frame: i32, duration: i32) -> Self {
        Self {
            start_frame: current_frame,
            duration,
        }
    }

    /// Return the raw signed frame anchor.
    #[inline]
    pub const fn start_frame(self) -> i32 {
        self.start_frame
    }

    /// Return the raw signed countdown duration.
    #[inline]
    pub const fn duration(self) -> i32 {
        self.duration
    }

    /// Re-anchor this timer and replace its countdown duration.
    #[inline]
    pub fn start(&mut self, current_frame: i32, duration: i32) {
        self.start_frame = current_frame;
        self.duration = duration;
    }

    /// Return the signed remaining time using wrapping frame subtraction.
    #[inline]
    pub fn remaining(self, current_frame: i32) -> i32 {
        if self.start_frame == PAUSED_START_FRAME {
            self.duration
        } else {
            let elapsed = current_frame.wrapping_sub(self.start_frame);
            if elapsed < self.duration {
                self.duration.wrapping_sub(elapsed)
            } else {
                0
            }
        }
    }

    /// Expiry is the exact zero boundary.
    #[inline]
    pub fn expired(self, current_frame: i32) -> bool {
        self.remaining(current_frame) == 0
    }

    /// Freeze the current remainder in `duration`.
    #[inline]
    pub fn pause(&mut self, current_frame: i32) {
        if self.start_frame != PAUSED_START_FRAME {
            self.duration = self.remaining(current_frame);
            self.start_frame = PAUSED_START_FRAME;
        }
    }

    /// Resume a paused countdown from `current_frame`.
    #[inline]
    pub fn resume(&mut self, current_frame: i32) {
        if self.start_frame == PAUSED_START_FRAME {
            self.start_frame = current_frame;
        }
    }

    #[inline]
    pub const fn is_paused(self) -> bool {
        self.start_frame == PAUSED_START_FRAME
    }
}
