# synthonyx-kit-tracing

W3C trace-context propagation and an opinionated `tracing` integration
for the Synthonyx Kit. The underlying `CorrelationId` and `SpanId` types
live in [`synthonyx-kit-core`](../synthonyx-kit-core); this crate adds
HTTP propagation and a tokio task-local current trace.

## Contents

- `TraceContext { correlation, parent, sampled }`.
- `CURRENT_TRACE` — a `tokio::task_local!` for the current task's trace
  context. Set at the inbound API edge so dispatches inside the scope
  can read it without explicit plumbing.
- `extract_w3c(headers: &HeaderMap) -> Option<TraceContext>` — parse the
  W3C `traceparent` header. Only version `00` is accepted; malformed
  headers return `None` rather than panicking, so services exposed to
  untrusted clients cannot be crashed via header injection.
- `inject_w3c(ctx, &mut HeaderMap)` — format and insert.
- `TRACEPARENT` constant — the W3C header name.

```rust
use http::HeaderMap;
use synthonyx_kit_tracing::{TraceContext, inject_w3c, extract_w3c};

let ctx = TraceContext::new(
    synthonyx_kit_core::CorrelationId([0u8; 16]),
    None,
    true,
);
let mut headers = HeaderMap::new();
inject_w3c(&ctx, &mut headers).unwrap();
assert_eq!(extract_w3c(&headers), Some(ctx));
```

Coverage includes a proptest fuzz-style test that the parser never panics
on arbitrary input — see
[`proptest_w3c.rs`](tests/proptest_w3c.rs).
