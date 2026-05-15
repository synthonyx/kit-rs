//! Compliance contract tests for W3C traceparent propagation.
//!
//! References:
//! - DORA Art. 17 (incident attribution — correlation must propagate
//!   across service boundaries).
//!
//! Contracts:
//! - Valid `traceparent` headers round-trip through `inject_w3c` /
//!   `extract_w3c`.
//! - Headers with version != 00 are rejected (forward-compat: future
//!   versions must be added explicitly, not silently consumed).
//! - Malformed headers return `None` rather than panicking, so a service
//!   exposed to untrusted clients cannot be crashed via header injection.
//! - The zero span-id decodes to `parent = None` (W3C-defined sentinel).

use http::HeaderMap;
use http::header::HeaderValue;
use synthonyx_kit_core::SpanId;
use synthonyx_kit_tracing::{TRACEPARENT, TraceContext, extract_w3c, inject_w3c};

fn parse_raw(raw: &'static str) -> Option<TraceContext> {
    let mut headers = HeaderMap::new();
    headers.insert(&TRACEPARENT, HeaderValue::from_static(raw));
    extract_w3c(&headers)
}

#[test]
fn art_17_round_trip_for_canonical_header() {
    let raw = "00-0a1b2c3d4e5f60718293a4b5c6d7e8f9-1122334455667788-01";
    let ctx = parse_raw(raw).expect("canonical header parses");
    let mut out = HeaderMap::new();
    inject_w3c(&ctx, &mut out).unwrap();
    let formatted = out.get(&TRACEPARENT).unwrap().to_str().unwrap();
    assert_eq!(formatted, raw);
}

#[test]
fn art_17_unknown_version_is_rejected() {
    // ff = future version not understood by this implementation.
    assert!(parse_raw("ff-0a1b2c3d4e5f60718293a4b5c6d7e8f9-1122334455667788-01").is_none());
    assert!(parse_raw("01-0a1b2c3d4e5f60718293a4b5c6d7e8f9-1122334455667788-01").is_none());
}

#[test]
fn art_17_missing_header_returns_none() {
    assert!(extract_w3c(&HeaderMap::new()).is_none());
}

#[test]
fn art_17_malformed_headers_return_none_not_panic() {
    // Each of these would crash a naive parser; none must.
    let cases: &[&'static str] = &[
        "",
        "00",
        "00-",
        "00-zzzz",
        "00-0a1b2c3d4e5f60718293a4b5c6d7e8f9",
        "00-0a1b2c3d4e5f60718293a4b5c6d7e8f9-1122334455667788",
        "00-0a1b2c3d4e5f60718293a4b5c6d7e8f9-1122334455667788-",
        "00-0a1b2c3d4e5f60718293a4b5c6d7e8f9-1122334455667788-01-extra",
        "00--1122334455667788-01",
        // Wrong-length trace-id (31 chars):
        "00-0a1b2c3d4e5f60718293a4b5c6d7e8f-1122334455667788-01",
        // Non-hex characters:
        "00-zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz-1122334455667788-01",
    ];
    for raw in cases {
        let parsed = parse_raw(raw);
        assert!(
            parsed.is_none(),
            "expected None for {raw:?}, got {parsed:?}"
        );
    }
}

#[test]
fn art_17_zero_span_id_decodes_as_none() {
    let ctx = parse_raw("00-0a1b2c3d4e5f60718293a4b5c6d7e8f9-0000000000000000-01")
        .expect("zero span id is valid");
    assert_eq!(ctx.parent, None);
}

#[test]
fn art_17_non_zero_span_id_decodes_as_some() {
    let ctx = parse_raw("00-0a1b2c3d4e5f60718293a4b5c6d7e8f9-0000000000000001-01")
        .expect("non-zero span id is valid");
    assert_eq!(ctx.parent, Some(SpanId([0, 0, 0, 0, 0, 0, 0, 1])));
}

#[test]
fn art_17_sampled_flag_bit_is_lsb() {
    let unsampled = parse_raw("00-0a1b2c3d4e5f60718293a4b5c6d7e8f9-1122334455667788-00").unwrap();
    assert!(!unsampled.sampled);
    let sampled = parse_raw("00-0a1b2c3d4e5f60718293a4b5c6d7e8f9-1122334455667788-01").unwrap();
    assert!(sampled.sampled);
    // Other flag bits are documented as reserved; bit 0 still controls
    // sampling.
    let sampled_with_other_bits =
        parse_raw("00-0a1b2c3d4e5f60718293a4b5c6d7e8f9-1122334455667788-03").unwrap();
    assert!(sampled_with_other_bits.sampled);
}
