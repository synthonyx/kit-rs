//! W3C trace-context propagation for the Synthonyx Kit.
//!
//! Provides:
//! - [`TraceContext`]: trace + span ids and the sampled flag.
//! - [`CURRENT_TRACE`]: a tokio task-local current trace context.
//! - [`extract_w3c`] / [`inject_w3c`]: parse/format the W3C `traceparent` HTTP
//!   header for cross-service propagation.
//!
//! The underlying [`CorrelationId`] / [`SpanId`] types live in
//! [`synthonyx_kit_core`].
#![deny(missing_docs, unsafe_code, rust_2018_idioms)]

use http::HeaderMap;
use http::header::{HeaderName, HeaderValue, InvalidHeaderValue};
use synthonyx_kit_core::{CorrelationId, SpanId};

/// The W3C `traceparent` header name.
pub const TRACEPARENT: HeaderName = HeaderName::from_static("traceparent");

/// W3C-compatible trace context carried across service boundaries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TraceContext {
    /// The correlation (trace) id.
    pub correlation: CorrelationId,
    /// The parent span id, if any.
    pub parent: Option<SpanId>,
    /// Whether the trace is sampled.
    pub sampled: bool,
}

impl TraceContext {
    /// Construct a `TraceContext` from its components.
    pub const fn new(correlation: CorrelationId, parent: Option<SpanId>, sampled: bool) -> Self {
        Self {
            correlation,
            parent,
            sampled,
        }
    }
}

tokio::task_local! {
    /// The current task-local [`TraceContext`].
    ///
    /// Set this via `CURRENT_TRACE.scope(ctx, fut).await` at the inbound edge
    /// of your service (e.g. in an HTTP middleware) so that every dispatch
    /// inside the scope can read the correlation id without explicit plumbing.
    pub static CURRENT_TRACE: TraceContext;
}

/// Parse a W3C `traceparent` header from `headers`, if present and valid.
///
/// Only version `00` is accepted. Malformed headers return `None`.
pub fn extract_w3c(headers: &HeaderMap) -> Option<TraceContext> {
    let raw = headers.get(&TRACEPARENT)?.to_str().ok()?;
    parse_traceparent(raw)
}

/// Format a `traceparent` header from `ctx` and insert it into `headers`.
pub fn inject_w3c(ctx: &TraceContext, headers: &mut HeaderMap) -> Result<(), InvalidHeaderValue> {
    let value = format_traceparent(ctx);
    let hv = HeaderValue::from_str(&value)?;
    headers.insert(&TRACEPARENT, hv);
    Ok(())
}

fn parse_traceparent(raw: &str) -> Option<TraceContext> {
    let mut parts = raw.split('-');
    let version = parts.next()?;
    let trace_id = parts.next()?;
    let parent_id = parts.next()?;
    let flags = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    if version != "00" {
        return None;
    }
    let trace_bytes = decode_hex::<16>(trace_id)?;
    let parent_bytes = decode_hex::<8>(parent_id)?;
    let flag_byte = u8::from_str_radix(flags, 16).ok()?;
    let parent = if parent_bytes == [0u8; 8] {
        None
    } else {
        Some(SpanId(parent_bytes))
    };
    Some(TraceContext {
        correlation: CorrelationId(trace_bytes),
        parent,
        sampled: flag_byte & 0x01 != 0,
    })
}

fn format_traceparent(ctx: &TraceContext) -> String {
    let span = ctx.parent.unwrap_or(SpanId::ZERO);
    let flags = u8::from(ctx.sampled);
    format!("00-{}-{}-{flags:02x}", ctx.correlation, span)
}

fn decode_hex<const N: usize>(s: &str) -> Option<[u8; N]> {
    if s.len() != N * 2 {
        return None;
    }
    let mut out = [0u8; N];
    let bytes = s.as_bytes();
    for i in 0..N {
        let hi = hex_nibble(bytes[i * 2])?;
        let lo = hex_nibble(bytes[i * 2 + 1])?;
        out[i] = (hi << 4) | lo;
    }
    Some(out)
}

fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_context() -> TraceContext {
        TraceContext {
            correlation: CorrelationId([
                0x0a, 0x1b, 0x2c, 0x3d, 0x4e, 0x5f, 0x60, 0x71, 0x82, 0x93, 0xa4, 0xb5, 0xc6, 0xd7,
                0xe8, 0xf9,
            ]),
            parent: Some(SpanId([0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88])),
            sampled: true,
        }
    }

    #[test]
    fn round_trip_w3c() {
        let ctx = sample_context();
        let mut headers = HeaderMap::new();
        inject_w3c(&ctx, &mut headers).unwrap();

        let raw = headers.get(&TRACEPARENT).unwrap().to_str().unwrap();
        assert_eq!(
            raw,
            "00-0a1b2c3d4e5f60718293a4b5c6d7e8f9-1122334455667788-01"
        );

        let parsed = extract_w3c(&headers).unwrap();
        assert_eq!(parsed, ctx);
    }

    #[test]
    fn rejects_unknown_version() {
        let mut headers = HeaderMap::new();
        headers.insert(
            &TRACEPARENT,
            HeaderValue::from_static("ff-0a1b2c3d4e5f60718293a4b5c6d7e8f9-1122334455667788-01"),
        );
        assert!(extract_w3c(&headers).is_none());
    }

    #[test]
    fn missing_header_returns_none() {
        assert!(extract_w3c(&HeaderMap::new()).is_none());
    }

    #[test]
    fn unsampled_parses_correctly() {
        let mut headers = HeaderMap::new();
        headers.insert(
            &TRACEPARENT,
            HeaderValue::from_static("00-00000000000000000000000000000001-0000000000000002-00"),
        );
        let ctx = extract_w3c(&headers).unwrap();
        assert!(!ctx.sampled);
        assert_eq!(ctx.parent, Some(SpanId([0, 0, 0, 0, 0, 0, 0, 2])));
    }

    #[test]
    fn zero_parent_decodes_as_none() {
        let mut headers = HeaderMap::new();
        headers.insert(
            &TRACEPARENT,
            HeaderValue::from_static("00-00000000000000000000000000000001-0000000000000000-01"),
        );
        let ctx = extract_w3c(&headers).unwrap();
        assert_eq!(ctx.parent, None);
    }
}
