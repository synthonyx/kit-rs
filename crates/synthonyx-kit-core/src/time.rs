//! Time sources, injected via `Config::Time` so audit timestamps and latency
//! measurements are reproducible in tests.

use std::sync::atomic::{AtomicU64, Ordering};

/// A nanosecond-precision wall-clock timestamp anchored at the UNIX epoch.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnixNanos(pub u128);

/// A nanosecond-precision monotonic timestamp with no defined epoch — only
/// suitable for measuring elapsed time.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MonotonicNanos(pub u128);

/// A pluggable source of timestamps.
pub trait TimeSource: Send + Sync + 'static {
    /// Wall-clock time, suitable for audit and externally-visible timestamps.
    fn now(&self) -> UnixNanos;
    /// Monotonic time, suitable for latency measurement.
    fn monotonic(&self) -> MonotonicNanos;
}

/// The system clock, backed by `std::time`.
///
/// `monotonic()` is anchored at process start so values fit in `u128`
/// comfortably.
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemClock;

impl TimeSource for SystemClock {
    #[allow(clippy::disallowed_methods)]
    fn now(&self) -> UnixNanos {
        let d = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time is before UNIX epoch");
        UnixNanos(d.as_nanos())
    }

    fn monotonic(&self) -> MonotonicNanos {
        use std::sync::OnceLock;
        use std::time::Instant;
        static EPOCH: OnceLock<Instant> = OnceLock::new();
        let epoch = EPOCH.get_or_init(Instant::now);
        MonotonicNanos(epoch.elapsed().as_nanos())
    }
}

/// A deterministic, manually-advanced clock for tests.
///
/// Both clocks are stored as `AtomicU64` and advance together via
/// [`Self::advance_nanos`].
#[derive(Debug)]
pub struct MockClock {
    wall: AtomicU64,
    mono: AtomicU64,
}

impl MockClock {
    /// Construct a `MockClock` starting at the given wall-clock and monotonic
    /// nanosecond values.
    pub const fn new(wall: u64, mono: u64) -> Self {
        Self {
            wall: AtomicU64::new(wall),
            mono: AtomicU64::new(mono),
        }
    }

    /// Advance both clocks by `delta` nanoseconds.
    pub fn advance_nanos(&self, delta: u64) {
        self.wall.fetch_add(delta, Ordering::Relaxed);
        self.mono.fetch_add(delta, Ordering::Relaxed);
    }
}

impl Default for MockClock {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

impl TimeSource for MockClock {
    fn now(&self) -> UnixNanos {
        UnixNanos(u128::from(self.wall.load(Ordering::Relaxed)))
    }

    fn monotonic(&self) -> MonotonicNanos {
        MonotonicNanos(u128::from(self.mono.load(Ordering::Relaxed)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_clock_starts_at_zero_and_advances() {
        let clock = MockClock::default();
        assert_eq!(clock.now(), UnixNanos(0));
        assert_eq!(clock.monotonic(), MonotonicNanos(0));
        clock.advance_nanos(1_000);
        assert_eq!(clock.now(), UnixNanos(1_000));
        assert_eq!(clock.monotonic(), MonotonicNanos(1_000));
    }

    #[test]
    fn system_clock_monotonic_is_monotonic() {
        let clock = SystemClock;
        let a = clock.monotonic();
        let b = clock.monotonic();
        assert!(b >= a);
    }
}
