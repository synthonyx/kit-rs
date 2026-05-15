//! Property-based tests for the W3C `traceparent` parser/formatter.
//!
//! Two properties are checked:
//! - Round-trip: for any valid `TraceContext`, formatting and then
//!   re-parsing yields the same context.
//! - Robustness: the parser never panics on arbitrary `String` input.
//!   This is the security-relevant property — a service that takes
//!   inbound `traceparent` headers from the public internet must never
//!   crash on malicious input.

use http::HeaderMap;
use http::header::HeaderValue;
use proptest::prelude::*;
use synthonyx_kit_core::{CorrelationId, SpanId};
use synthonyx_kit_tracing::{TRACEPARENT, TraceContext, extract_w3c, inject_w3c};

proptest! {
    #[test]
    fn round_trip_any_valid_context(
        trace_bytes: [u8; 16],
        span_bytes: [u8; 8],
        has_parent: bool,
        sampled: bool,
    ) {
        let ctx = TraceContext {
            correlation: CorrelationId(trace_bytes),
            parent: if has_parent && span_bytes != [0u8; 8] {
                Some(SpanId(span_bytes))
            } else {
                None
            },
            sampled,
        };

        let mut headers = HeaderMap::new();
        inject_w3c(&ctx, &mut headers).unwrap();
        let parsed = extract_w3c(&headers).expect("inject/extract round-trip");
        prop_assert_eq!(parsed, ctx);
    }

    #[test]
    fn parser_never_panics_on_arbitrary_input(raw in ".*") {
        let Ok(value) = HeaderValue::from_str(&raw) else {
            // Inputs that can't even be a header value are out of scope.
            return Ok(());
        };
        let mut headers = HeaderMap::new();
        headers.insert(&TRACEPARENT, value);
        // Must return Some(_) or None — never panic.
        let _ = extract_w3c(&headers);
    }
}
