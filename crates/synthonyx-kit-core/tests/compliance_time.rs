//! Compliance contract tests for `TimeSource`, `MockClock`, `SystemClock`.
//!
//! References:
//! - DORA Art. 17 (incident reporting attribution requires accurate,
//!   non-decreasing timestamps that can be reproduced in tests).
//!
//! `Config::Time` is the injection point; without an injectable clock,
//! audit timestamps could not be deterministically reviewed during a
//! regulator-led incident reconstruction.

use synthonyx_kit_core::{MockClock, MonotonicNanos, SystemClock, TimeSource, UnixNanos};

#[test]
fn art_17_mock_clock_is_deterministic() {
    let clock = MockClock::new(1_000, 2_000);
    assert_eq!(clock.now(), UnixNanos(1_000));
    assert_eq!(clock.monotonic(), MonotonicNanos(2_000));
    clock.advance_nanos(500);
    assert_eq!(clock.now(), UnixNanos(1_500));
    assert_eq!(clock.monotonic(), MonotonicNanos(2_500));
}

#[test]
fn art_17_mock_clock_default_starts_at_zero() {
    let clock = MockClock::default();
    assert_eq!(clock.now(), UnixNanos(0));
    assert_eq!(clock.monotonic(), MonotonicNanos(0));
}

#[test]
fn art_17_system_clock_monotonic_never_decreases() {
    let clock = SystemClock;
    let mut prev = clock.monotonic();
    for _ in 0..1_000 {
        let next = clock.monotonic();
        assert!(next >= prev, "SystemClock monotonic decreased");
        prev = next;
    }
}

#[test]
fn art_17_system_clock_wall_clock_is_sane() {
    let clock = SystemClock;
    let now = clock.now();
    // After 2020-01-01 (sanity floor).
    assert!(now.0 > 1_577_836_800_000_000_000);
    // Before 3000-01-01 (sanity ceiling).
    assert!(now.0 < 32_503_680_000_000_000_000);
}

#[test]
fn time_source_is_object_safe_via_static_dispatch() {
    fn use_time<T: TimeSource>(t: &T) -> UnixNanos {
        t.now()
    }
    let _ = use_time(&MockClock::new(7, 7));
    let _ = use_time(&SystemClock);
}
