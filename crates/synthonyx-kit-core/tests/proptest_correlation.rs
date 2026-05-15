//! Property-based tests for `CorrelationId` and `SpanId` hex round-trip.
//!
//! The lower-case hex `Display` impl is the wire format consumed by
//! `synthonyx-kit-tracing`'s W3C `traceparent` formatter; if it diverged
//! from parseable hex, cross-service correlation would silently break.

use proptest::prelude::*;
use synthonyx_kit_core::{CorrelationId, SpanId};

fn nibble(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        b'a'..=b'f' => b - b'a' + 10,
        _ => unreachable!("Display emits 0-9a-f only"),
    }
}

fn decode_hex_16(s: &str) -> [u8; 16] {
    assert_eq!(s.len(), 32, "trace-id must be 32 hex chars");
    let bytes = s.as_bytes();
    let mut out = [0u8; 16];
    for i in 0..16 {
        out[i] = (nibble(bytes[i * 2]) << 4) | nibble(bytes[i * 2 + 1]);
    }
    out
}

fn decode_hex_8(s: &str) -> [u8; 8] {
    assert_eq!(s.len(), 16, "span-id must be 16 hex chars");
    let bytes = s.as_bytes();
    let mut out = [0u8; 8];
    for i in 0..8 {
        out[i] = (nibble(bytes[i * 2]) << 4) | nibble(bytes[i * 2 + 1]);
    }
    out
}

proptest! {
    #[test]
    fn correlation_id_hex_round_trip(bytes: [u8; 16]) {
        let id = CorrelationId(bytes);
        let s = id.to_string();
        prop_assert_eq!(s.len(), 32);
        prop_assert!(s.chars().all(|c| c.is_ascii_hexdigit() && (c.is_ascii_digit() || c.is_ascii_lowercase())));
        prop_assert_eq!(decode_hex_16(&s), bytes);
    }

    #[test]
    fn span_id_hex_round_trip(bytes: [u8; 8]) {
        let id = SpanId(bytes);
        let s = id.to_string();
        prop_assert_eq!(s.len(), 16);
        prop_assert_eq!(decode_hex_8(&s), bytes);
    }
}
